#include <stdio.h>
// Function prototype declaration
unsigned long long factorial(int n);
int main() {
  int number;
  unsigned long long result;
  number = 3;
  result = 0;
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
// Function to calculate the factorial of a number
unsigned long long factorial(int n) {
  if (n == 0) {
    return 1; // Base case: factorial of 0 is 1
  } else {
    return n * factorial(n - 1); // Recursive case
  }
}
