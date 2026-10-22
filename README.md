# ⚡ XeroKey

**A fast, Rust-powered ZIP password recovery toolkit.**

Recover access to your password-protected ZIP archives using wordlists or automatic numeric password generation.

Built for speed. Designed for the terminal. Powered by Rust.

---

## ⚡ Installation

Open your terminal and run:

```bash
git clone https://github.com/XeroNox/XeroKey.git
cd XeroKey
bash install.sh
```

Once installation finishes, launch XeroKey:

```bash
xerokey --help
```

---

## 🚀 Features

- **High performance** — built with Rust for efficient execution.
- **Parallel processing** — checks password candidates concurrently.
- **Wordlist recovery** — searches through passwords from a text file.
- **Automatic numeric generation** — generates numeric password candidates automatically.
- **Custom password lengths** — choose the minimum and maximum lengths for numeric searches.
- **Progress tracking** — monitor the recovery process in your terminal.
- **Lightweight workflow** — straightforward command-line usage.

---

## 🧰 How to Use

### 1. Recover Using a Wordlist

Have a list of possible passwords? Pass the ZIP archive and wordlist to XeroKey:

```bash
xerokey archive.zip wordlist.txt
```

Your wordlist should contain one candidate password per line:

```text
password123
welcome123
example2026
mytestpassword
```

XeroKey checks the supplied candidates against the archive. If the correct password is present and the archive is supported, it may recover the password.

### 2. Generate Numeric Passwords Automatically

Don't have a wordlist? Use automatic numeric generation:

```bash
xerokey archive.zip --auto
```

To search numeric passwords with lengths from one to six digits:

```bash
xerokey archive.zip --auto --min-length 1 --max-length 6
```

To search all four-digit numeric passwords, including those beginning with zero:

```bash
xerokey archive.zip --auto --min-length 4 --max-length 4
```

**Tip:** Choose a realistic length range. Larger search spaces can take considerably longer to complete.

### 3. View Available Options

```bash
xerokey --help
```

---

## 🔢 Understanding Numeric Password Recovery

Numeric password searches can grow quickly as the password length increases.

| Password Length | Possible Combinations |
|---:|---:|
| 3 digits | 1,000 |
| 4 digits | 10,000 |
| 5 digits | 100,000 |
| 6 digits | 1 million |
| 8 digits | 100 million |
| 10 digits | 10 billion |
| 12 digits | 1 trillion |

These figures represent the total possible combinations, not a guarantee of how long recovery will take.

The actual duration depends on your device, the archive's encryption, and the speed of password verification.

---

## ⚙️ Requirements

- A supported operating system and processor architecture.
- A compatible XeroKey binary or a Rust toolchain for building from source.
- A ZIP archive you are authorized to access.
- A wordlist if you choose wordlist-based recovery.

On Termux, if Rust is required to build XeroKey, install it with:

```bash
pkg update
pkg install rust git
```

Then run the installation steps again.

---

## 💡 Which Mode Should You Use?

| Situation | Recommended Mode |
|---|---|
| You have a list of likely passwords | Wordlist mode |
| You suspect the password contains only digits | Automatic numeric mode |
| You know the password length | Set a specific length range |
| You suspect letters or symbols are included | Use a suitable wordlist containing those candidates |

Choosing the right candidates is often more important than simply increasing the number of attempts.

---

## ⚠️ Important Limitations

- Recovery is not guaranteed.
- Wordlist mode cannot find a password that is absent from the supplied candidates.
- Numeric mode searches only numeric passwords within the selected range.
- Long or complex passwords may require an impractical amount of time to recover.
- Compatibility depends on the ZIP archive's encryption and format.
- Performance varies depending on your device and the archive being tested.

---

## 📜 License

XeroKey is released under the **MIT License**.

You are free to use, copy, modify, merge, publish, distribute, sublicense, and sell copies of the software, subject to the conditions of the MIT License.

See the [`LICENSE`](LICENSE) file for the complete license text.

---

## ⚖️ Legal Notice

XeroKey is intended for legitimate password recovery, authorized security testing, and educational purposes.

Only use XeroKey on files you own or have explicit permission to access. Unauthorized access to another person's files or data may violate applicable laws.

The software is provided **"AS IS," without warranty of any kind**, as described in the MIT License. The authors and contributors are not responsible for misuse of the software.

**Use XeroKey responsibly and only with proper authorization.**
