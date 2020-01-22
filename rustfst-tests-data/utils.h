#ifndef MY_UTILS_H
#define MY_UTILS_H

float custom_random_float() {
    int num = rand() % 100 + 1;
    num = num - 50;

    return float(num) / 10.0;
}

#endif