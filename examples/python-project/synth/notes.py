#!/usr/bin/env python3
# @handwritten


def frequency(midi_note: int) -> float:
    """Equal-tempered frequency for a MIDI note number."""
    return 440.0 * 2 ** ((midi_note - 69) / 12)


if __name__ == "__main__":
    print(frequency(60))
