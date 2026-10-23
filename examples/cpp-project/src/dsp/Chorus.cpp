/*
 * Chorus effect.
 */
// @handwritten

#include <vector>

float chorus(const std::vector<float>& buffer, std::size_t i) {
    float wet = buffer[i] * 0.5f;
    return buffer[i] + wet;
}
