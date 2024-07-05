#include <stdio.h>
// Function prototype declaration
unsigned long long factorial(int n);
// This one should stay!
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
// THIS ONE SHOULD STAY
// Function to calculate the factorial of a number
// THIS MIGHT BE REMOVED!
unsigned long long factorial(int n) {
  if (n == 0) {
    // THIS ONE SHOULD BE FILTERED!
    return 1; // Base case: factorial of 0 is 1
  } else {
    return n * factorial(n - 1); // Recursive case
  }
}
