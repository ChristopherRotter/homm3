/* A forwarder (E17 M1 T8): the GOG Heroes3.exe imports DirectDrawCreate from xdd.dll, its
   DDrawCompat wrapper, which does not run under Wine; this xdd.dll hands the call to Wine's own
   ddraw (xdd.def). Built by setup.sh with mingw-w64. */
#include <windows.h>

BOOL WINAPI DllMain(HINSTANCE instance, DWORD reason, LPVOID reserved)
{
    (void)instance;
    (void)reason;
    (void)reserved;
    return TRUE;
}
