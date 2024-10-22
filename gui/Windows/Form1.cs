using System.Diagnostics;

namespace LanKM
{
    public partial class LanKM : Form
    {
        private Process? mProc;
        public LanKM()
        {
            InitializeComponent();
            update_button_text();
        }

        void update_button_text()
        {
            if (mProc == null)
            {
                startStopButton.Text = "Start";
            }
            else
            {
                startStopButton.Text = "Stop";
            }
        }

        private async void update_log()
        {
            while (mProc != null && !mProc.HasExited)
            {
                var buffer = new char[256];
                var count = await mProc.StandardOutput.ReadAsync(buffer);

                var text = new string(buffer[..count]);
                textBox1.AppendText(text);
            }

            mProc = null;
            update_button_text();
        }

        private void StartServer()
        {
            var startInfo = new ProcessStartInfo("ping")
            {
                UseShellExecute = false,
                Arguments = "google.com -t",
                RedirectStandardOutput = true,
                RedirectStandardInput = true,
                CreateNoWindow = true
            };

            try
            {
                mProc = Process.Start(startInfo);
            }
            catch (Exception e)
            {
                Console.WriteLine(e);
            }

            update_log();
        }

        private void StopServer()
        {
            if (mProc == null)
            {
                return;
            }

            // FIXME: This should be a terminate, not a kill, but CloseMainWindow
            // doesn't work on things that don't have windows and as far as my quick
            // searches go that's the only way to send a terminate signal
            mProc.Kill();
        }

        private void startStopButton_Click(object sender, EventArgs _e)
        {
            if (mProc == null)
            {
                StartServer();
            } else
            {
                StopServer();
            }
            update_button_text();
        }
    }
}
