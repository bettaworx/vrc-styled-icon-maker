@echo off

set TYPE=bold

node ./SVG2PNG.js ./.assets/phosphor-icons/SVGs/%TYPE%/ ./input_images_%TYPE%/
python ./img_process.py -i ./input_images_%TYPE%/ -o ./../phosphor-icons/%TYPE%/

pause