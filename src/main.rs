use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;
use std::process;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

fn cetak_banner() {
    let banner = "
========================================
     _____  _____  _______      __
    / ____|/ ____|/ ____\\ \\    / /
   | (___ | |    | |     \\ \\  / / 
    \\___ \\| |    | |      \\ \\/ /  
    ____) | |____| |____   \\  /   
   |_____/ \_____|\_____|   \\/    
                                  
       [ ADVANCED FUZZER ENGINE v3.0 ]
       Author: sdev (2026)
========================================
    ";
    println!("{}", banner);
}

fn target(d: &str) -> bool {
    if d.contains("ADMIN") && d.contains("bypass_token") && d.len() > 40 {
        panic!("CRASH_DETECTED_MEM_FAULT");
    }
    true
}

fn muat_corpus_dan_dictionary() -> Vec<String> {
    let mut kamus = vec![
        "A".to_string(), "B".to_string(), "!".to_string(), "@".to_string(),
        "#".to_string(), "$".to_string(), "%".to_string(), "x".to_string(),
        "y".to_string(), "\x00".to_string(), "\n".to_string(), "ADMIN".to_string(),
        "bypass_token".to_string(), "A".repeat(50)
    ];

    if Path::new("dictionary.txt").exists() {
        if let Ok(konten) = fs::read_to_string("dictionary.txt") {
            for baris in konten.lines() {
                if !baris.trim().is_empty() {
                    kamus.push(baris.trim().to_string());
                }
            }
        }
    }

    if Path::new("corpus").is_dir() {
        if let Ok(entri_dir) = fs::read_dir("corpus") {
            for entri in entri_dir.flatten() {
                if let Ok(konten) = fs::read_to_string(entri.path()) {
                    if !konten.trim().is_empty() {
                        kamus.push(konten);
                    }
                }
            }
        }
    }

    kamus
}

fn mutasi(seed: u32, kamus: &[String]) -> String {
    let mut rng = seed;
    rng ^= rng << 13;
    rng ^= rng >> 17;
    rng ^= rng << 5;

    if kamus.is_empty() {
        return "test_payload".to_string();
    }

    let idx_awal = (rng as usize) % kamus.len();
    let mut hasil = kamus[idx_awal].clone();

    let jumlah_mutasi = (rng % 5) + 1;
    for i in 0..jumlah_mutasi {
        let idx_tambahan = ((rng + i) as usize) % kamus.len();
        hasil.push_str(&kamus[idx_tambahan]);
    }

    if hasil.len() > 500 {
        hasil.truncate(500);
    }

    hasil
}

fn simpan_crash(payload: &str, iterasi: u64, error_msg: &str) {
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("crash_report.txt") {
        let data = format!(
            "--- CRASH LOG ---\nIterasi: {}\nPayload: {:?}\nError: {}\n-----------------\n\n",
            iterasi, payload, error_msg
        );
        let _ = file.write_all(data.as_bytes());
    }
}

fn main() {
    cetak_banner();

    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Cara Penggunaan: {} <timeout_dalam_detik>", args[0]);
        println!("Contoh         : {} 60 (untuk run selama 1 menit)", args[0]);
        process::exit(1);
    }

    let detik_timeout: u64 = args[1].parse().unwrap_or_else(|_| {
        println!("[!] Gagal membaca nilai timeout. Default diatur ke 30 detik.");
        30
    });

    println!("[*] Membaca database kata...");
    let kamus = Arc::new(muat_corpus_dan_dictionary());
    println!("[+] Total entry terkumpul (Default + Dictionary + Corpus): {}", kamus.len());

    let jumlah_thread = num_cpus::get();
    println!("[*] Menjalankan {} thread sekaligus...", jumlah_thread);
    println!("[*] Batas waktu pengerjaan: {} detik\n", detik_timeout);

    let berjalan = Arc::new(AtomicBool::new(true));
    let total_percobaan = Arc::new(AtomicU64::new(0));
    let mulai = Instant::now();

    let berjalan_timer = Arc::clone(&berjalan);
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(detik_timeout));
        if berjalan_timer.swap(false, Ordering::Relaxed) {
            println!("\n\n[-] WAKTU HABIS! Batas waktu {} detik telah tercapai.", detik_timeout);
            println!("[-] Fuzzer dihentikan otomatis secara aman.");
            process::exit(0);
        }
    });

    let mut info_threads = vec![];

    for t_id in 0..jumlah_thread {
        let berjalan_clone = Arc::clone(&berjalan);
        let percobaan_clone = Arc::clone(&total_percobaan);
        let kamus_clone = Arc::clone(&kamus);

        let handle = thread::spawn(move || {
            let mut seed = 123456789 + (t_id as u32 * 777);
            let mut lokal_iterasi: u64 = 0;

            while berjalan_clone.load(Ordering::Relaxed) {
                lokal_iterasi += 1;
                percobaan_clone.fetch_add(1, Ordering::Relaxed);

                seed = seed.wrapping_add(lokal_iterasi as u32);
                let payload = mutasi(seed, &kamus_clone);

                let hasil_target = std::panic::catch_unwind(|| {
                    target(&payload);
                });

                if hasil_target.is_err() {
                    if berjalan_clone.swap(false, Ordering::Relaxed) {
                        let durasi = mulai.elapsed();
                        let total_iter = percobaan_clone.load(Ordering::Relaxed);

                        println!("\n{}", "!".repeat(50));
                        println!("[!] CRASH DITEMUKAN OLEH THREAD-{}!", t_id);
                        println!("[!] Total Iterasi Semua Thread : {}", total_iter);
                        println!("[!] Payload Pemicu             : {:?}", payload);
                        println!("[!] Error                      : Panic / Memory Fault");
                        println!("[!] Waktu Eksekusi             : {:.4?} detik", durasi.as_secs_f64());
                        println!("[*] Menyimpan payload ke 'crash_report.txt'...");
                        println!("{}", "!".repeat(50));

                        simpan_crash(&payload, total_iter, "Panic / Memory Fault");
                        process::exit(0);
                    }
                }

                if t_id == 0 && lokal_iterasi % 500000 == 0 {
                    let total = percobaan_clone.load(Ordering::Relaxed);
                    let ops_per_detik = total as f64 / mulai.elapsed().as_secs_f64();
                    print!(
                        "[*] Hasil Tes: {} payload | Kecepatan: {:.0} exec/sec\r",
                        total, ops_per_detik
                    );
                    let _ = io::stdout().flush();
                }
            }
        });
        info_threads.push(handle);
    }

    for handle in info_threads {
        let _ = handle.join();
    }
}
}
