// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
// DRIVER BRIDGE
// User-mode interface for communicating with the kernel driver via IOCTLs.
// ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

using System;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;
using System.Security.Principal;

namespace LeylineHSA
{
    public class DriverBridge : IDisposable
    {
        // {A1B2C3D4-E5F6-4A2D-B3C4-D5E6F7A8B9C0}
        public static readonly Guid LEYLINE_INTERFACE_GUID = new Guid(0xA1B2C3D4, 0xE5F6, 0x4A2D, 0xB3, 0xC4, 0xD5, 0xE6, 0xF7, 0xA8, 0xB9, 0xC0);

        // IOCTLs (FILE_DEVICE_UNKNOWN = 0x22, METHOD_BUFFERED = 0, FILE_ANY_ACCESS = 0)
        public const uint IOCTL_LEYLINE_SET_CONFIG = 0x00222000;
        public const uint IOCTL_LEYLINE_GET_STATUS = 0x00222004;
        public const uint IOCTL_LEYLINE_MAP_BUFFER = 0x00222008;
        public const uint IOCTL_LEYLINE_MAP_PARAMS = 0x0022200C;

        private SafeFileHandle _handle;

        public bool IsConnected => _handle != null && !_handle.IsInvalid;

        public bool Connect()
        {
            // In a real HSA, we'd use SetupDiGetClassDevs / SetupDiEnumDeviceInterfaces
            // to find the symbolic link. For now, we'll use a placeholder or
            // the expected symbolic link name if known.
            // Typically: \\\\.\\LeylineAudio

            _handle = CreateFile(
                "\\\\.\\LeylineAudio",
                FileAccess.GenericRead | FileAccess.GenericWrite,
                FileShare.None,
                IntPtr.Zero,
                FileMode.Open,
                FileAttributes.Normal,
                IntPtr.Zero);

            return IsConnected;
        }

        public uint GetStatus()
        {
            if (!IsConnected) return 0xFFFFFFFF;

            uint status = 0;
            uint bytesReturned = 0;

            bool success = DeviceIoControlInt(
                _handle,
                IOCTL_LEYLINE_GET_STATUS,
                IntPtr.Zero, 0,
                out status, sizeof(uint),
                out bytesReturned,
                IntPtr.Zero);

            return success ? status : 0xFFFFFFFF;
        }

        public IntPtr MapBuffer()
        {
            if (!IsConnected) return IntPtr.Zero;

            IntPtr userPtr = IntPtr.Zero;
            uint bytesReturned = 0;

            bool success = DeviceIoControlPtr(
                _handle,
                IOCTL_LEYLINE_MAP_BUFFER,
                IntPtr.Zero, 0,
                out userPtr, (uint)IntPtr.Size,
                out bytesReturned,
                IntPtr.Zero);

            return success ? userPtr : IntPtr.Zero;
        }

        public IntPtr MapParams()
        {
            if (!IsConnected) return IntPtr.Zero;

            IntPtr userPtr = IntPtr.Zero;
            uint bytesReturned = 0;

            bool success = DeviceIoControlPtr(
                _handle,
                IOCTL_LEYLINE_MAP_PARAMS,
                IntPtr.Zero, 0,
                out userPtr, (uint)IntPtr.Size,
                out bytesReturned,
                IntPtr.Zero);

            return success ? userPtr : IntPtr.Zero;
        }

        public void Dispose()
        {
            _handle?.Dispose();
        }

        #region P/Invoke

        [Flags]
        private enum FileAccess : uint
        {
            GenericRead = 0x80000000,
            GenericWrite = 0x40000000
        }

        [Flags]
        private enum FileShare : uint
        {
            None = 0x00000000,
            Read = 0x00000001,
            Write = 0x00000002
        }

        private enum FileMode : uint
        {
            Open = 3
        }

        [Flags]
        private enum FileAttributes : uint
        {
            Normal = 0x00000080
        }

        [DllImport("kernel32.dll", SetLastError = true, CharSet = CharSet.Auto)]
        private static extern SafeFileHandle CreateFile(
            string lpFileName,
            FileAccess dwDesiredAccess,
            FileShare dwShareMode,
            IntPtr lpSecurityAttributes,
            FileMode dwCreationDisposition,
            FileAttributes dwFlagsAndAttributes,
            IntPtr hTemplateFile);

        [DllImport("kernel32.dll", EntryPoint = "DeviceIoControl", SetLastError = true)]
        private static extern bool DeviceIoControlInt(
            SafeFileHandle hDevice,
            uint dwIoControlCode,
            IntPtr lpInBuffer,
            uint nInBufferSize,
            out uint lpOutBuffer,
            uint nOutBufferSize,
            out uint lpBytesReturned,
            IntPtr lpOverlapped);

        [DllImport("kernel32.dll", EntryPoint = "DeviceIoControl", SetLastError = true)]
        private static extern bool DeviceIoControlPtr(
            SafeFileHandle hDevice,
            uint dwIoControlCode,
            IntPtr lpInBuffer,
            uint nInBufferSize,
            out IntPtr lpOutBuffer,
            uint nOutBufferSize,
            out uint lpBytesReturned,
            IntPtr lpOverlapped);

        #endregion
    }
}
