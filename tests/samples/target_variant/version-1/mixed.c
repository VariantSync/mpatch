#include <stdio.h>
unsigned long long factorial(int n);
int main() {
  int number;
  unsigned long long res;
  printf("Enter a positive integer: ");
  scanf("%d", &number);
  if (number < 0) {
    printf("Factorial of a negative number doesn't exist.\n");
  } else {
    //
    //
    //
    //
    //
    //
    //
    //
    //
    res = factorial(number);
    printf("Factorial of %d is %llu\n", number, res);
  }
  return 0;
}
unsigned long long factorial(int n) {
  if (n == 0) {
    return 1; // Base case: factorial of 0 is 1
  } else {
    return n * factorial(n - 1); // Recursive case
  }
}
