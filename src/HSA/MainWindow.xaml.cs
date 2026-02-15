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
        private IntPtr _bufferPtr;
        private double[] _audioHistory = new double[300]; // 300 points for the graph

        [StructLayout(LayoutKind.Sequential)]
        struct SharedParameters
        {
            public int master_gain_bits;
            public int peak_l_bits;
            public int peak_r_bits;
            public long qpc_frequency;
            public long render_start_qpc;
            public long capture_start_qpc;
            public uint buffer_size;
            public uint byte_rate;
        }

        [DllImport("kernel32.dll")]
        private static extern bool QueryPerformanceCounter(out long lpPerformanceCount);

        public MainWindow()
        {
            this.InitializeComponent();

            // Initialize graph history
            for (int i = 0; i < _audioHistory.Length; i++) _audioHistory[i] = 60; // Baseline (bottom)

            if (_bridge.Connect())
            {
                _paramsPtr = _bridge.MapParams();
                _bufferPtr = _bridge.MapBuffer();
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
            if (_paramsPtr != IntPtr.Zero && _bufferPtr != IntPtr.Zero)
            {
                SharedParameters* p = (SharedParameters*)_paramsPtr;
                long start = System.Threading.Interlocked.Read(ref p->render_start_qpc);
                long freq = System.Threading.Interlocked.Read(ref p->qpc_frequency);
                uint size = p->buffer_size;
                uint rate = p->byte_rate;

                float peakL = 0;
                float peakR = 0;
                float currentSample = 0;

                if (start > 0 && freq > 0 && size > 0 && rate > 0)
                {
                    long now;
                    QueryPerformanceCounter(out now);
                    double elapsed = (double)(now - start) / freq;
                    long bytes = (long)(elapsed * rate);
                    long offset = bytes % size;
                    offset &= ~7; // Align to 8 bytes (Stereo Float32)

                    // Read a small window (e.g., 256 samples) to find peak
                    int windowSamples = 256;
                    float* buffer = (float*)_bufferPtr;

                    for (int i = 0; i < windowSamples; i++)
                    {
                        long readPos = (offset + (i * 8)) % size;
                        float l = buffer[readPos / 4];     // Float index
                        float r = buffer[(readPos / 4) + 1];

                        if (Math.Abs(l) > peakL) peakL = Math.Abs(l);
                        if (Math.Abs(r) > peakR) peakR = Math.Abs(r);

                        if (i == 0) currentSample = l; // Visualize first sample of window
                    }
                }

                MeterL.Value = Math.Clamp(peakL * 100, 0, 100);
                MeterR.Value = Math.Clamp(peakR * 100, 0, 100);

                // Update Graph History
                Array.Copy(_audioHistory, 1, _audioHistory, 0, _audioHistory.Length - 1);

                // Visualize the current sample (simple oscilloscope)
                double y = 30 - (Math.Clamp(currentSample, -1, 1) * 30);
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
                int bits = BitConverter.SingleToInt32Bits((float)e.NewValue);
                System.Threading.Interlocked.Exchange(ref p->master_gain_bits, bits);
            }
        }
    }
}
