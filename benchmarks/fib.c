int fib(int n) { return n < 2 ? 1 : fib(n-1) + fib(n-2); }
int main() { return fib(20); }