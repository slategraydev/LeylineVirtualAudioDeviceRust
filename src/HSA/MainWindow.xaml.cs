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
        private DispatcherTimer _timer;
        private IntPtr _paramsPtr;

        [StructLayout(LayoutKind.Sequential)]
        struct SharedParameters
        {
            public float master_gain;
            public float peak_l;
            public float peak_r;
        }

        public MainWindow()
        {
            this.InitializeComponent();
            if (_bridge.Connect())
            {
                _paramsPtr = _bridge.MapParams();
                StatusText.Text = "Connected to Leyline Driver";
                
                _timer = new DispatcherTimer();
                _timer.Interval = TimeSpan.FromMilliseconds(33);
                _timer.Tick += (s, e) => UpdateUI();
                _timer.Start();
            }
            else
            {
                StatusText.Text = "Driver not found.";
            }
        }

        private unsafe void UpdateUI()
        {
            if (_paramsPtr != IntPtr.Zero)
            {
                SharedParameters* p = (SharedParameters*)_paramsPtr;
                MeterL.Value = Math.Clamp(p->peak_l * 100, 0, 100);
                MeterR.Value = Math.Clamp(p->peak_r * 100, 0, 100);
            }
        }

        private unsafe void GainSlider_ValueChanged(object sender, Microsoft.UI.Xaml.Controls.Primitives.RangeBaseValueChangedEventArgs e)
        {
            if (_paramsPtr != IntPtr.Zero)
            {
                SharedParameters* p = (SharedParameters*)_paramsPtr;
                p->master_gain = (float)e.NewValue;
            }
        }
    }
}
