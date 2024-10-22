namespace LanKM
{
    partial class LanKM
    {
        /// <summary>
        ///  Required designer variable.
        /// </summary>
        private System.ComponentModel.IContainer components = null;

        /// <summary>
        ///  Clean up any resources being used.
        /// </summary>
        /// <param name="disposing">true if managed resources should be disposed; otherwise, false.</param>
        protected override void Dispose(bool disposing)
        {
            if (disposing && (components != null))
            {
                components.Dispose();
            }
            base.Dispose(disposing);
        }

        #region Windows Form Designer generated code

        /// <summary>
        ///  Required method for Designer support - do not modify
        ///  the contents of this method with the code editor.
        /// </summary>
        private void InitializeComponent()
        {
            startStopButton = new Button();
            textBox1 = new TextBox();
            SuspendLayout();
            // 
            // startStopButton
            // 
            startStopButton.Location = new Point(397, 12);
            startStopButton.Name = "startStopButton";
            startStopButton.Size = new Size(75, 23);
            startStopButton.TabIndex = 0;
            startStopButton.Text = "button1";
            startStopButton.UseVisualStyleBackColor = true;
            startStopButton.Click += startStopButton_Click;
            // 
            // textBox1
            // 
            textBox1.Enabled = false;
            textBox1.Location = new Point(12, 41);
            textBox1.Multiline = true;
            textBox1.Name = "textBox1";
            textBox1.Size = new Size(460, 208);
            textBox1.TabIndex = 1;
            // 
            // LanKM
            // 
            AutoScaleDimensions = new SizeF(7F, 15F);
            AutoScaleMode = AutoScaleMode.Font;
            ClientSize = new Size(484, 261);
            Controls.Add(textBox1);
            Controls.Add(startStopButton);
            Name = "LanKM";
            Text = "LanKM";
            ResumeLayout(false);
            PerformLayout();
        }

        #endregion

        private Button startStopButton;
        private TextBox textBox1;
    }
}
