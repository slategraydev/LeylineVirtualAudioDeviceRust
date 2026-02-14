// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

using Microsoft.UI.Xaml;
using System;
using System.Runtime.InteropServices;
using Microsoft.Win32.SafeHandles;

namespace LeylineHSA
{
    public sealed partial class MainWindow : Window
    {
        private DriverBridge _bridge = new DriverBridge();

        public MainWindow()
        {
            this.InitializeComponent();
            CheckDriverStatus();
        }

        private void RefreshButton_Click(object sender, RoutedEventArgs e)
        {
            CheckDriverStatus();
        }

        private void CheckDriverStatus()
        {
            if (_bridge.Connect())
            {
                uint status = _bridge.GetStatus();
                StatusText.Text = $"Connected. Status: 0x{status:X8}";
            }
            else
            {
                StatusText.Text = "Driver not found. Ensure Leyline Audio Driver is installed.";
            }
        }
    }
}
