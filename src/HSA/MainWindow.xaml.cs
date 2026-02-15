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
        private double[] _audioHistory = new double[300]; // 300 points for the graph

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
            
            // Initialize graph history
            for (int i = 0; i < _audioHistory.Length; i++) _audioHistory[i] = 60; // Baseline (bottom)

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
                float peak = Math.Max(p->peak_l, p->peak_r);
                
                MeterL.Value = Math.Clamp(p->peak_l * 100, 0, 100);
                MeterR.Value = Math.Clamp(p->peak_r * 100, 0, 100);

                // Update Graph History
                // Shift left
                Array.Copy(_audioHistory, 1, _audioHistory, 0, _audioHistory.Length - 1);
                
                // Add new value (Height is 60, so 60 is bottom, 0 is top)
                double y = 60 - (Math.Clamp(peak, 0, 1) * 60);
                _audioHistory[_audioHistory.Length - 1] = y;

                // Redraw Line
                var collection = new Microsoft.UI.Xaml.Media.PointCollection();
                for (int i = 0; i < _audioHistory.Length; i++)
                {
                    collection.Add(new Windows.Foundation.Point(i, _audioHistory[i]));
                }
                WaveformLine.Points = collection;
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
