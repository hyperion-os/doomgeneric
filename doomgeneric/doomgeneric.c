#include <stdio.h>
#include <string.h>

#include "m_argv.h"

#include "doomgeneric.h"

uint32_t* DG_ScreenBuffer = 0;

void M_FindResponseFile(void);
void D_DoomMain (void);


void doomgeneric_Create(int argc, char **argv)
{
	// FILE *file;

	// file = fopen("file1.txt", "w");
	// if (file == NULL) {
	// 	printf("Error opening file.\n");
	// 	return;
	// }
	// fprintf(file, "0123456789\n");
	// fprintf(file, "abcdefghij\n");
	// fseek(file, 0, SEEK_SET);
	// fprintf(file, "ABC");
	// fclose(file);

	// file = fopen("file2.txt", "w");
	// if (file == NULL) {
	// 	printf("Error opening file.\n");
	// 	return;
	// }
	// fprintf(file, "0123456789\n");
	// fprintf(file, "abcdefghij\n");
	// fseek(file, 4, SEEK_SET);
	// fprintf(file, "ABC");
	// fseek(file, 1, SEEK_CUR);
	// fprintf(file, "D");
	// fseek(file, -4, SEEK_END);
	// fprintf(file, "**");
	// if (fseek(file, 10, SEEK_END) != 0) {
	// 	printf("fseek(file, 10, SEEK_END) should have failed");
	// }
	// fclose(file);

	// file = fopen("file2.txt", "r");
	// if (file == NULL) {
	// 	printf("Error opening file.\n");
	// 	return;
	// }
	// char buf[64];
	// for (int i = 0; i<64; i++) { buf[i] = 0; }
	// fread(buf, 1, 11, file);
	// if (strncmp(buf, "0123ABC7D9\n", 12) != 0) {
	// 	printf("fread1 result unexpected: %s", buf);
	// }
	// fread(buf, 1, 11, file);
	// if (strncmp(buf, "abcdefg**j\n", 12) != 0) {
	// 	printf("fread2 result unexpected: %s", buf);
	// }
	// fclose(file);
	
	// return;

	char buf[64];
	for (int i = 0; i<64; i++) { buf[i] = 8; }
	int result = snprintf(buf, 64, "testing-%s", "123");
	printf("snprintf result: %d '%s'", result, buf);

	printf("STCFN%.3d", 33);
	
	// save arguments
    myargc = argc;
    myargv = argv;

	M_FindResponseFile();

	DG_ScreenBuffer = malloc(DOOMGENERIC_RESX * DOOMGENERIC_RESY * 4);

	DG_Init();

	D_DoomMain ();
}

