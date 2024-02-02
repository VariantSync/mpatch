#include <stdio.h>
// Function prototype declaration
unsigned long long factorial(int n);
int main() { return 0; }
// Function to calculate the factorial of a number
unsigned long long factorial(int n) {
  if (n == 0) {
    return 1; // Base case: factorial of 0 is 1
  } else {
    return n * factorial(n - 1); // Recursive case
  }
}
