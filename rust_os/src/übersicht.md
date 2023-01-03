# Was wir in main machen
In der Reihenfolge (✔ heißt das sollte eigentlich schon funktionieren)
- memory remap ✔
- stackpointer initialisieren ✔
- dbgu enablen (RXEN und TXEN setzen) ✔
- Die jump-Befehle in die ivt schreiben ✔
- Die exception handleradressen für dab, und, swi schreiben ✔
- Den AIC aufsetzen. 
    1. Version: Nichts
    2. Version: Alle interrupt-Quellen enabled mit prio 0 und src_type 0 und mit einem default handler.
- Den interrupt handler an der src 1 mit prio 7 und dem typ 0 gesetzt. Der handler wird nie ausgeführt (oder zumindest der print in dem handler)
- Bei der dbgu die Interrupts enablen
- Den sys timer interrupt enablen
- und den interval auf 32768 (1s) setzen
- Endlosschleife (Inhalt gerade egal, weil der interrupt handler sowieso nicht aufgerufen wird) ✔