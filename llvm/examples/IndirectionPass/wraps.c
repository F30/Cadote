#include <stdio.h>

void foo() {
  printf("hello world\n");
}

int main() {
  foo();
  return 0;
}

void wrapper(void (*func)()) {
  (*func)();
}
