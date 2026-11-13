"""Capture a specific Windows window without relying on foreground order."""

from __future__ import annotations

import ctypes
from ctypes import wintypes

from PIL import Image


PW_RENDERFULLCONTENT = 0x00000002
DIB_RGB_COLORS = 0
BI_RGB = 0

user32 = ctypes.WinDLL("user32", use_last_error=True)
gdi32 = ctypes.WinDLL("gdi32", use_last_error=True)


class BitmapInfoHeader(ctypes.Structure):
    _fields_ = [
        ("size", wintypes.DWORD),
        ("width", wintypes.LONG),
        ("height", wintypes.LONG),
        ("planes", wintypes.WORD),
        ("bit_count", wintypes.WORD),
        ("compression", wintypes.DWORD),
        ("size_image", wintypes.DWORD),
        ("x_pels_per_meter", wintypes.LONG),
        ("y_pels_per_meter", wintypes.LONG),
        ("colors_used", wintypes.DWORD),
        ("colors_important", wintypes.DWORD),
    ]


class RgbQuad(ctypes.Structure):
    _fields_ = [
        ("blue", wintypes.BYTE),
        ("green", wintypes.BYTE),
        ("red", wintypes.BYTE),
        ("reserved", wintypes.BYTE),
    ]


class BitmapInfo(ctypes.Structure):
    _fields_ = [
        ("header", BitmapInfoHeader),
        ("colors", RgbQuad * 1),
    ]


user32.GetWindowRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
user32.GetWindowRect.restype = wintypes.BOOL
user32.GetClientRect.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.RECT)]
user32.GetClientRect.restype = wintypes.BOOL
user32.ClientToScreen.argtypes = [wintypes.HWND, ctypes.POINTER(wintypes.POINT)]
user32.ClientToScreen.restype = wintypes.BOOL
user32.GetWindowDC.argtypes = [wintypes.HWND]
user32.GetWindowDC.restype = wintypes.HDC
user32.ReleaseDC.argtypes = [wintypes.HWND, wintypes.HDC]
user32.ReleaseDC.restype = ctypes.c_int
user32.PrintWindow.argtypes = [wintypes.HWND, wintypes.HDC, wintypes.UINT]
user32.PrintWindow.restype = wintypes.BOOL

gdi32.CreateCompatibleDC.argtypes = [wintypes.HDC]
gdi32.CreateCompatibleDC.restype = wintypes.HDC
gdi32.CreateCompatibleBitmap.argtypes = [wintypes.HDC, ctypes.c_int, ctypes.c_int]
gdi32.CreateCompatibleBitmap.restype = wintypes.HANDLE
gdi32.SelectObject.argtypes = [wintypes.HDC, wintypes.HANDLE]
gdi32.SelectObject.restype = wintypes.HANDLE
gdi32.DeleteObject.argtypes = [wintypes.HANDLE]
gdi32.DeleteObject.restype = wintypes.BOOL
gdi32.DeleteDC.argtypes = [wintypes.HDC]
gdi32.DeleteDC.restype = wintypes.BOOL
gdi32.GetDIBits.argtypes = [
    wintypes.HDC,
    wintypes.HANDLE,
    wintypes.UINT,
    wintypes.UINT,
    ctypes.c_void_p,
    ctypes.POINTER(BitmapInfo),
    wintypes.UINT,
]
gdi32.GetDIBits.restype = ctypes.c_int


def capture_client_image(hwnd: int) -> Image.Image:
    window_rect = wintypes.RECT()
    client_rect = wintypes.RECT()
    if not user32.GetWindowRect(hwnd, ctypes.byref(window_rect)):
        raise ctypes.WinError(ctypes.get_last_error())
    if not user32.GetClientRect(hwnd, ctypes.byref(client_rect)):
        raise ctypes.WinError(ctypes.get_last_error())

    client_origin = wintypes.POINT(client_rect.left, client_rect.top)
    if not user32.ClientToScreen(hwnd, ctypes.byref(client_origin)):
        raise ctypes.WinError(ctypes.get_last_error())

    width = window_rect.right - window_rect.left
    height = window_rect.bottom - window_rect.top
    client_width = client_rect.right - client_rect.left
    client_height = client_rect.bottom - client_rect.top
    client_x = client_origin.x - window_rect.left
    client_y = client_origin.y - window_rect.top

    window_dc = user32.GetWindowDC(hwnd)
    if not window_dc:
        raise ctypes.WinError(ctypes.get_last_error())
    memory_dc = gdi32.CreateCompatibleDC(window_dc)
    if not memory_dc:
        user32.ReleaseDC(hwnd, window_dc)
        raise ctypes.WinError(ctypes.get_last_error())
    bitmap = gdi32.CreateCompatibleBitmap(window_dc, width, height)
    if not bitmap:
        gdi32.DeleteDC(memory_dc)
        user32.ReleaseDC(hwnd, window_dc)
        raise ctypes.WinError(ctypes.get_last_error())

    previous_object = gdi32.SelectObject(memory_dc, bitmap)
    try:
        if not user32.PrintWindow(hwnd, memory_dc, PW_RENDERFULLCONTENT):
            raise ctypes.WinError(ctypes.get_last_error())

        info = BitmapInfo()
        info.header.size = ctypes.sizeof(BitmapInfoHeader)
        info.header.width = width
        info.header.height = -height
        info.header.planes = 1
        info.header.bit_count = 32
        info.header.compression = BI_RGB
        pixels = (ctypes.c_ubyte * (width * height * 4))()
        rows = gdi32.GetDIBits(
            memory_dc,
            bitmap,
            0,
            height,
            pixels,
            ctypes.byref(info),
            DIB_RGB_COLORS,
        )
        if rows != height:
            raise ctypes.WinError(ctypes.get_last_error())

        window_image = Image.frombuffer(
            "RGB", (width, height), pixels, "raw", "BGRX", 0, 1
        )
        return window_image.crop(
            (client_x, client_y, client_x + client_width, client_y + client_height)
        )
    finally:
        gdi32.SelectObject(memory_dc, previous_object)
        gdi32.DeleteObject(bitmap)
        gdi32.DeleteDC(memory_dc)
        user32.ReleaseDC(hwnd, window_dc)
