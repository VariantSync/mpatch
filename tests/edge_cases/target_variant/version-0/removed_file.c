#include <stdio.h>
// Function prototype declaration
unsigned long long factorial(int n);
int main() {
  int number;
  unsigned long long result;
  // Ask the user for input
  printf("Enter a positive integer: ");
  scanf("%d", &number);
  // Check if the user has entered a negative integer
  if (number < 0) {
    printf("Factorial of a negative number doesn't exist.\n");
  } else {
    // Calculate factorial
    result = factorial(number);
    // Display the result
    printf("Factorial of %d is %llu\n", number, result);
  }
  return 0;
}
