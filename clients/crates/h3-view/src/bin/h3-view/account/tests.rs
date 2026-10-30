//! The account's key and its file (E17 M5 T7).

use std::io::Cursor;

use ed25519_dalek::{Signature, VerifyingKey};
use h3_protocol::PROTOCOL_VERSION;
use h3_protocol::account::{CHALLENGE_BYTES, NONCE_BYTES};
use h3_protocol::wire::ServerMessage;
use h3_view::remote::Note;

use super::*;
use crate::net;

/// The derivation's test vector (`docs/lanes/client/NOTES.md`, "The live client"; the server
/// lane's, computed independently with `argon2-cffi` and PyNaCl).
const ALICE_SEED: &str = "a6d07dbd0bc53fa424145cbb87da7a85b49b8905044250b913940d4871c2ba69";
const ALICE_PUBLIC: &str = "b17dbb2796b53c1c636e9a5981d6424ac5ee096b89b47223567831f435a8af14";

const ADDRESS: &str = "ws://127.0.0.1:7000";
const NONCE: [u8; NONCE_BYTES] = [7; NONCE_BYTES];
const CHALLENGE: [u8; CHALLENGE_BYTES] = [9; CHALLENGE_BYTES];

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("h3-view-account-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn args(s: &str) -> Vec<String> {
    s.split(' ').map(str::to_owned).collect()
}

#[test]
fn the_key_derivation_matches_its_test_vector() {
    let seed = derive("alice", "correct horse battery staple").unwrap();
    assert_eq!(hex(&seed), ALICE_SEED);
    assert_eq!(hex(&public(&seed)), ALICE_PUBLIC);
    assert_eq!(login_name("Alice").unwrap(), "alice", "lowercased");
    assert!(login_name("al").unwrap_err().contains("no login name"));
}

#[test]
fn the_options_name_the_login_and_the_files() {
    let parsed = Options::parse(&args("Alice --change --accounts a --pins p")).unwrap();
    assert_eq!(
        parsed,
        Options {
            login: "alice".to_owned(),
            change: true,
            accounts: PathBuf::from("a"),
            pins: PathBuf::from("p"),
        }
    );
    let plain = Options::parse(&args("bob")).unwrap();
    let files = (plain.change, plain.accounts, plain.pins);
    assert_eq!(files, (false, ACCOUNTS.into(), "pins.txt".into()));
    for (bad, why) in [
        ("--change", "needs the login name"),
        ("bob --pins", "needs a value"),
        ("bob --what", "unknown option"),
        ("bob carol", "unexpected"),
        ("b", "no login name"),
    ] {
        let error = Options::parse(&args(bad)).unwrap_err();
        assert!(error.contains(why), "{bad}: {error}");
    }
}

#[test]
fn the_key_is_kept_and_a_password_change_leaves_it_due_at_the_servers_pinned() {
    let dir = scratch("keep");
    let options = Options {
        login: "alice".to_owned(),
        change: false,
        accounts: dir.join("keys/accounts.txt"),
        pins: dir.join("pins.txt"),
    };
    let kept = keep(
        &options,
        &mut Cursor::new("correct horse battery staple\r\n"),
    )
    .unwrap();
    assert_eq!(kept[0], format!("alice's key {ALICE_PUBLIC}"));
    let text = std::fs::read_to_string(&options.accounts).unwrap();
    assert!(text.starts_with("# h3-view's account keys"), "{text}");
    assert!(
        text.ends_with(&format!("alice key {ALICE_SEED}\n")),
        "{text}"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(&options.accounts)
            .unwrap()
            .permissions()
            .mode();
        assert_eq!(mode & 0o777, 0o600, "the owner's alone");
        // Made readable by others meanwhile, the next write makes it the owner's again (E17 M5
        // review R1-4).
        let others = std::fs::Permissions::from_mode(0o644);
        std::fs::set_permissions(&options.accounts, others).unwrap();
    }
    // The change: the key kept here is the current one; the new password alone is read.
    let pins = format!("ws://x {}\nwss://y {}\n", hex(&[1; 32]), hex(&[2; 32]));
    std::fs::write(&options.pins, pins).unwrap();
    let change = Options {
        change: true,
        ..options.clone()
    };
    let changed = keep(&change, &mut Cursor::new("new\n")).unwrap();
    assert_eq!(
        changed[1..3],
        [
            "re-keyed at the next login at ws://x",
            "re-keyed at the next login at wss://y"
        ]
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(&options.accounts).unwrap().permissions();
        assert_eq!(mode.mode() & 0o777, 0o600, "the owner's alone again");
    }
    let accounts = read_accounts(&options.accounts).unwrap();
    let before = account::from_hex(ALICE_SEED);
    assert_eq!(accounts.due("alice", "wss://y"), before);
    assert_ne!(accounts.key("alice"), before);
    // The same password again, or none, changes nothing.
    let same = keep(&change, &mut Cursor::new("new\n")).unwrap_err();
    assert!(same.contains("the current one"), "{same}");
    let none = keep(&options, &mut Cursor::new("\n")).unwrap_err();
    assert!(none.contains("no password"), "{none}");
    // A bad file is an error with its line.
    std::fs::write(&options.accounts, "alice key 00\n").unwrap();
    let bad = keep(&options, &mut Cursor::new("x\n")).unwrap_err();
    assert!(bad.ends_with("line 1"), "{bad}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A client welcomed at [`ADDRESS`] by a server key, as the I/O thread hands the welcome over.
fn welcomed() -> Client {
    let server = SigningKey::from_bytes(&[42; 32]);
    let signed = account::welcome_message(ADDRESS, &NONCE, &CHALLENGE);
    let welcome = ServerMessage::Welcome {
        protocol: PROTOCOL_VERSION,
        server: "test".to_owned(),
        id: ADDRESS.to_owned(),
        challenge: hex(&CHALLENGE),
        key: hex(&server.verifying_key().to_bytes()),
        signature: hex(&server.sign(&signed).to_bytes()),
    };
    let mut client = Client::new(ADDRESS, NONCE, None);
    client.receive(welcome, net::verify).unwrap();
    client
}

/// Whether `signature` (hex) verifies `message` by `key` (hex).
fn verifies(key: &str, message: &[u8], signature: &str) -> bool {
    let key = VerifyingKey::from_bytes(&account::from_hex(key).unwrap()).unwrap();
    let signature = Signature::from_bytes(&account::from_hex(signature).unwrap());
    key.verify_strict(message, &signature).is_ok()
}

#[test]
fn a_connection_logs_in_by_the_key_kept_and_re_keys_where_an_earlier_one_is_due() {
    let dir = scratch("login");
    let account = Account {
        login: "alice".to_owned(),
        register: None,
        accounts: dir.join("accounts.txt"),
    };
    let missing = account.login(&mut welcomed(), ADDRESS).unwrap_err();
    assert!(missing.contains("h3-view account alice"), "{missing}");
    let (old, new) = ([5; SEED_BYTES], [6; SEED_BYTES]);
    let mut accounts = Accounts::default();
    accounts.set("alice", old);
    accounts.change("alice", new, &[ADDRESS, "ws://other"]);
    write_accounts(&account.accounts, &accounts).unwrap();
    // By the earlier key due here, signed for the address dialed and the welcome's challenge.
    let mut client = welcomed();
    let ClientMessage::Login { key, signature, .. } = account.login(&mut client, ADDRESS).unwrap()
    else {
        panic!("a login");
    };
    assert_eq!(key, hex(&public(&old)));
    let signed = account::login_message(ADDRESS, &CHALLENGE, "alice", None);
    assert!(verifies(&key, &signed, &signature));
    assert!(account.login(&mut client, ADDRESS).is_err(), "one login");
    let answer = || ServerMessage::Account {
        seq: 1,
        display: "Alice".to_owned(),
        listed: false,
    };
    let notes = client.receive(answer(), net::verify).unwrap();
    let [
        Note::Send(ClientMessage::Rekey {
            key,
            by_old,
            by_new,
            ..
        }),
    ] = &notes[..]
    else {
        panic!("the re-key: {notes:?}");
    };
    let signed = account::rekey_message(ADDRESS, &CHALLENGE, "alice", &public(&new));
    assert!(verifies(&hex(&public(&old)), &signed, by_old));
    assert!(verifies(key, &signed, by_new));
    // Re-keyed here: the earlier key is due at the other server alone.
    account.done(ADDRESS).unwrap();
    let accounts = read_accounts(&account.accounts).unwrap();
    assert_eq!(accounts.due("alice", ADDRESS), None);
    assert_eq!(accounts.due("alice", "ws://other"), Some(old));
    // Elsewhere, by the account's key, registering it.
    let register = Account {
        register: Some("Alice".to_owned()),
        ..account.clone()
    };
    let ClientMessage::Login {
        key,
        signature,
        display,
        ..
    } = register.login(&mut welcomed(), ADDRESS).unwrap()
    else {
        panic!("a login");
    };
    assert_eq!(
        (key.as_str(), display.as_deref()),
        (hex(&public(&new)).as_str(), Some("Alice"))
    );
    let signed = account::login_message(ADDRESS, &CHALLENGE, "alice", Some("Alice"));
    assert!(verifies(&key, &signed, &signature));
    std::fs::remove_dir_all(&dir).unwrap();
}
