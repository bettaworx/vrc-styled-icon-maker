# How to make icons

### 0. Getting started
First you'll need to install these softwares/packages to run icon maker scripts.

- Node.js
- Python

### 1. Install dependencies

Node.js:
```js
npm install sharp
```

Python:
```python
pip install Pillow opencv-python numpy
```

### 2. Run

To make png file from svg file:
```js
node ./SVG2PNG.js <input_folder> <output_folder>
```

To make styled icons from png file:
```python
python ./img_process.py -i <input_folder> -o <output_folder>
```