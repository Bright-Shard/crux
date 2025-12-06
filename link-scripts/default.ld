/* TODO: Find a way to make LLD load this section as read-only */
SECTIONS {
	.crux.ini : ALIGN(64) {
		__crux_ini_start = .;
		*(.crux.ini);
		__crux_ini_end = .;
	}
  /*
  Leave this INSERT AFTER in - without it, LLD tries to make an ELF file that
  loads .crux.ini in *two* memory regions... which is obviously invalid, so LD
  just crashes lmao
  */
} INSERT AFTER .rodata;
