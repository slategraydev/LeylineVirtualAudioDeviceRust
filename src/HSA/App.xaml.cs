// Copyright (c) 2026 Randall Rosas (Slategray).
// All rights reserved.

using Microsoft.UI.Xaml;

namespace LeylineHSA
{
    public partial class App : Application
    {
        public App()
        {
            this.InitializeComponent();
        }

        protected override void OnLaunched(Microsoft.UI.Xaml.LaunchActivatedEventArgs args)
        {
            m_window = new MainWindow();
            m_window.Activate();
        }

        private Window m_window;
    }
}

