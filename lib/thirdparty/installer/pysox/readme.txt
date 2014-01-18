
Flac is used for convert .wav file into .flac file.
.wav is recorded by pyaudio. 
but .flac is needed by Google speech recognition api.

https://pypi.python.org/pypi/pysox

Required prerequisite are the development libraries of sox at version 14.3.x, i.e. the header files and libraries to link against.
Specifically you need sox.h in your include path and libsox.so and libsox.a in your link path. Pysox will not compile against any sox version prior to 14.3.0.