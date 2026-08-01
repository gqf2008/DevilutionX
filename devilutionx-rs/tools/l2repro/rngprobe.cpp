#include <cstdio>
#include "engine/random.hpp"
using namespace devilution;
int main() {
    SetRndSeed(68685319u);
    int a = GenerateRnd(6);
    int b = GenerateRnd(6);
    int c = GenerateRnd(37);
    int d = GenerateRnd(37);
    printf("%d %d %d %d\n", a, b, c, d);
    return 0;
}
