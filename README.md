# ShortCircuit - Zero Knowledge Circuit Development Tool for Circom

## Introduction
ShortCircuit is an open-source developer tool designed for writing and debugging zero-knowledge circuits using Circom. Our tool streamlines the development process with an innovative visual debugger, making it easier to design and test circuits.

## Features
Circom Language Support: Write circuits in the powerful and flexible Circom language.
Visual Debugger: Our tool's standout feature, offers real-time graphical representation and debugging of circuits.
User-Friendly Interface: Designed for both beginners and advanced users in the field of zero-knowledge proofs.


## Usage
To start using ShortCircuit: 
Navigate to http://production_url in your web browser to access the visual debugger interface.

Writing a Circuit
Create a new .circom file and write your circuit as you would in Circom. For example:

    template Multiplier() {
        signal input a, b;
        signal output c;
        c <== a * b;
    }
    component main = Multiplier();
  
## Debugging a Circuit
Load your .circom file into ShortCircuit.
Use the visual interface to set input values and observe the computation.
The debugger highlights active parts of the circuit and shows intermediate values.

---Screenshots

## Contributing
We welcome contributions! Please read our Contributing Guide for details on our code of conduct, and the process for submitting pull requests.

## License
This project is licensed under the MIT License - see the LICENSE.md file for details.

## Acknowledgments
- Circom Community
