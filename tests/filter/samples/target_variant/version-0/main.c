#include <stdio.h>
#define HELLO 1
// Function prototype declaration
unsigned long long factorial(int n);
int main() {
  int number;
  unsigned long long result;
  // This removal should stay as well!
  // Ask the user for input
  printf("Enter a positive integer: ");
  scanf("%d", &number);

#ifdef HELLO
  printf("Hello World\n");
#endif /* ifdef HELLO */

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
