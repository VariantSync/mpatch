#include <stdio.h>
// Function prototype declaration
unsigned long long factorial(int n);

// Function to calculate the factorial of a number
unsigned long long factorial(int n) {
  if (n == 0) {
    return 1; // Base case: factorial of 0 is 1
  } else {
    return n * factorial(n - 1); // Recursive case
  }
}

int main() {
  int number;
  unsigned long long res;
  // Ask the user for input
  printf("Enter a positive integer: ");
  scanf("%d", &number);
  // Check if the user has entered a negative integer
  if (number < 0) {
    printf("Factorial of a negative number doesn't exist.\n");
  } else {
    // Calculate factorial
    res = factorial(number);
    // Display the result
    printf("Factorial of %d is %llu\n", number, res);
  }
  return 0;
}
