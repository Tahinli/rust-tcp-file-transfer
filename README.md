[![Rust](https://github.com/Tahinli/rust-tcp-file-transfer/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/Tahinli/rust-tcp-file-transfer/actions/workflows/rust.yml)
# rust-tcp-file-transfer
TCP File Transfer Server and Client in Rust

**Usage**
> ./rust-tcp-file-transfer -h
>
![image](https://github.com/Tahinli/rust-tcp-file-transfer/assets/96421894/7370c9f7-e491-42de-bf81-8f16b8daa248)

**Examples**
> ./rust-tcp-file-transfer -sv -s -l ~/Desktop/cat.png
>
> ./rust-tcp-file-transfer -cl -r -l ~/Documents/

<img width="960" alt="image" src="https://github.com/user-attachments/assets/d0095f59-2489-4623-87f6-168e4fa52412">

**TO-DO List**

☑ Standard library only.

☑ File transfer over network.

☑ Remove memory limitations. [d42412c](https://github.com/Tahinli/rust-tcp-file-transfer/pull/1/commits/d42412c57d7d95672ba64b3e489b95f1c4b04a08)

☑ Bidirectional transfer. [b0531de](https://github.com/Tahinli/rust-tcp-file-transfer/commit/b0531deb257332f46fc76de16d3a17fb3b28acee)
