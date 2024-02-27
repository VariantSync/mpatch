#include <stdio.h>
int main() {
  int number;
  unsigned long long result;
  scanf("%d", &number);
  // Check if the user has entered a negative integer
  if (number < 0) {
    printf("Factorial of a negative number doesn't exist.\n");
  }
  return 0;
}
