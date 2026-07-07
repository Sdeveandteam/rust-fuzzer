# rust-fuzzer
**An advanced multi-threaded mutation-based fuzzer written in Rust**

Dibuat oleh: `sdev`  
License: MIT - Gratis. Pakai sepuasnya.

### **KENAPA TOOLS INI ADA?**
Tools fuzzer komersil kayak Burp Pro, Acunetix harganya puluhan juta/tahun.
Tools ini gw bikin GRATIS buat: Dev, Pentester, UMKM yg mau ngetes keamanan aplikasinya sendiri.
Ga perlu bayar. Ga perlu kenal gw. Cukup tau namanya `sdev`.

### **FITUR UTAMA**
1.  **Multi-threaded** → Nembak ribuan request/detik
2.  **Mutation-based** → Otomatis mutasi payload buat cari bypass
3.  **Auto Report** → Hasil crash/bypass langsung masuk `crash_report.txt`
4.  **Ringan** → Bisa jalan di Linux, VPS, bahkan Termux

### **CARA INSTALL & PAKAI**
```bash
# 1. Clone repo
git clone https://github.com/Sdeveandteam/rust-fuzzer.git
cd rust-fuzzer

# 2. Build
cargo build --release

# 3. Jalanin
./target/release/rust-fuzzer http://target.com
