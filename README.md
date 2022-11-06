# Betriebssysteme-WiSe22

## Für das bauen und ausführen auf Andorra
```$ make```

(Holt sich die Rust binarys zum compilieren von /home/mi/jbork läd aber die sonderfunktionen (nightly version, dependancys, etc) nochmal extra - das dauert leider ein wenig.)

## Bauen auf eigenem Gerät und ausführen auf Andorra:

- Download und install GNU Arm Embedded Toolchain (https://developer.arm.com/downloads/-/gnu-rm)  
(for linux just download and set PATH)

- install rust with rustup (https://www.rust-lang.org/tools/install)

- gehe in den Ordner rust_OS und baue das Projekt ($ cargo build)  
Cargo gibt nun Anweisungen für weitere installationen (e.g. nightly build, core build, etc)

- Nach erfolgreichem Kompilieren und Linken findet man im Ordner `target/armv4t-none-eabi/debug` die ELF-Datei.

- die ELF-Datei kann dann über Andorra in QEMU gestartet werden
    ```bash
    LD_LIBRARY_PATH=/usr/local/lib:/import/sage-7.4/local/lib/ && export LD_LIBRARY_PATH
    /home/mi/linnert/arm/bin/qemu-bsprak -kernel kernel
    ```

- Damit man die LED's sich anzeigen lassen kann:  
    ```bash
    /.../qemu-bsprak -kernel kernel -piotelnet
    telnet localhost 44444
    ```
## Ausführen auf eigenem Gerät
ToDo

