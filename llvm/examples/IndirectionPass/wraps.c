#include <stdio.h>

void foo_wrapped_() {
  printf("hello world\n");
}

int main() {
  foo_wrapped_();
  return 0;
}

void wrapper(void (*func)()) {
  (*func)();
}
