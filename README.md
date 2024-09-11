[![Rust](https://github.com/Tahinli/rust-tcp-file-transfer/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/Tahinli/rust-tcp-file-transfer/actions/workflows/rust.yml)
# rust-tcp-file-transfer
TCP File Transfer Server and Client in Rust

**Usage**
> ./rust-tcp-file-transfer -h
>
<img src=assets/help_menu.png>


**Examples**
> ./rust-tcp-file-transfer -sv -s -l ~/Desktop/cat.png
>
> ./rust-tcp-file-transfer -cl -r -l ~/Documents/

<img src=assets/example.png>

**TO-DO List**

☑ Standard library only.

☑ File transfer over network.

☑ Remove memory limitations. [d42412c](https://github.com/Tahinli/rust-tcp-file-transfer/pull/1/commits/d42412c57d7d95672ba64b3e489b95f1c4b04a08)

☑ Bidirectional transfer. [b0531de](https://github.com/Tahinli/rust-tcp-file-transfer/commit/b0531deb257332f46fc76de16d3a17fb3b28acee)
