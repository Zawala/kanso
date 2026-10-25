/*
 * Calls kanso_background_sync from a plain C process, as the iOS and Android
 * background jobs do: no Dart, no Flutter, no JVM. Built and run against the
 * library for this computer by scripts/mobile-background-check.sh.
 *
 * A new library has no account, so the answer is -1 ("not signed in"). A null
 * path is -5 ("internal"). Anything else is a failure of the entry itself.
 */
#define _DEFAULT_SOURCE
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

/* The entry point, as the app's Swift and Kotlin declare it. */
extern int32_t kanso_background_sync(const char *library_path, uint32_t budget_ms);

static int expect(const char *what, int32_t got, int32_t want) {
    if (got != want) {
        fprintf(stderr, "%s: got %d, wanted %d\n", what, (int)got, (int)want);
        return 1;
    }
    printf("%s: %d\n", what, (int)got);
    return 0;
}

int main(void) {
    const char *tmp = getenv("TMPDIR");
    char folder[256];
    snprintf(folder, sizeof folder, "%s/kanso-c-abi-XXXXXX", tmp != NULL && *tmp != 0 ? tmp : "/tmp");
    if (mkdtemp(folder) == NULL) {
        perror("mkdtemp");
        return 2;
    }
    char path[300];
    snprintf(path, sizeof path, "%s/library.db", folder);

    int failed = 0;
    failed |= expect("a new library, signed out", kanso_background_sync(path, 1000), -1);
    failed |= expect("the same library again", kanso_background_sync(path, 1000), -1);
    failed |= expect("a null path", kanso_background_sync(NULL, 1000), -5);
    failed |= expect("a path that is not a file", kanso_background_sync(folder, 1000), -5);

    char cleanup[400];
    snprintf(cleanup, sizeof cleanup, "rm -rf '%s'", folder);
    if (system(cleanup) != 0) {
        fprintf(stderr, "couldn't remove %s\n", folder);
    }
    return failed;
}
