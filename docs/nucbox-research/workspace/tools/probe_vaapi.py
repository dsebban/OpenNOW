#!/usr/bin/env python3
"""Query the installed VA-API driver. Does not decode video or alter settings."""
import ctypes as c, ctypes.util, json, os

va=c.CDLL(ctypes.util.find_library('va'))
drm=c.CDLL(ctypes.util.find_library('va-drm'))
drm.vaGetDisplayDRM.argtypes=[c.c_int]; drm.vaGetDisplayDRM.restype=c.c_void_p
va.vaInitialize.argtypes=[c.c_void_p,c.POINTER(c.c_int),c.POINTER(c.c_int)]
va.vaTerminate.argtypes=[c.c_void_p]
va.vaMaxNumProfiles.argtypes=[c.c_void_p]
va.vaMaxNumEntrypoints.argtypes=[c.c_void_p]
va.vaQueryConfigProfiles.argtypes=[c.c_void_p,c.POINTER(c.c_int),c.POINTER(c.c_int)]
va.vaQueryConfigEntrypoints.argtypes=[c.c_void_p,c.c_int,c.POINTER(c.c_int),c.POINTER(c.c_int)]
va.vaQueryVendorString.argtypes=[c.c_void_p]; va.vaQueryVendorString.restype=c.c_char_p
va.vaProfileStr.argtypes=[c.c_int]; va.vaProfileStr.restype=c.c_char_p
va.vaEntrypointStr.argtypes=[c.c_int]; va.vaEntrypointStr.restype=c.c_char_p
def check(status):
    if status: raise RuntimeError(f'VA-API status {status}')
fd=os.open('/dev/dri/renderD128',os.O_RDWR)
display=drm.vaGetDisplayDRM(fd); initialized=False
try:
    major=c.c_int(); minor=c.c_int()
    check(va.vaInitialize(display,c.byref(major),c.byref(minor))); initialized=True
    profiles=(c.c_int*va.vaMaxNumProfiles(display))(); count=c.c_int()
    check(va.vaQueryConfigProfiles(display,profiles,c.byref(count)))
    result=[]
    for profile in profiles[:count.value]:
        entries=(c.c_int*va.vaMaxNumEntrypoints(display))(); n=c.c_int()
        check(va.vaQueryConfigEntrypoints(display,profile,entries,c.byref(n)))
        result.append({'profile':va.vaProfileStr(profile).decode(),'entrypoints':[va.vaEntrypointStr(x).decode() for x in entries[:n.value]]})
    print(json.dumps({'device':'/dev/dri/renderD128','va_api':f'{major.value}.{minor.value}','driver':va.vaQueryVendorString(display).decode(),'profiles':result,'limitation':'Advertised profiles are not a decode performance or DMA-BUF export test.'},indent=2))
finally:
    if initialized: va.vaTerminate(display)
    os.close(fd)
