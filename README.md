# MidiMapper 

I use it to remap midi drum tracks to different drum maps, saving the result to a new midi file.

```
Usage: midimapper.exe [OPTIONS] <INPUT_FILEPATH> <OUTPUT_FILEPATH>

Arguments:
  <INPUT_FILEPATH>   Input midi file
  <OUTPUT_FILEPATH>  output path

Options:
  -i, --input-map-path <INPUT_MAP_PATH>    Input midi map // TODO: default to GM
  -o, --output-map-path <OUTPUT_MAP_PATH>  Output midi map
  -n, --track-number <TRACK_NUMBER>        Channel name to convert Channel number to convert
  -h, --help                               Print help
```