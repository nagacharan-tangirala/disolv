# Disolv

Disolv stands for **D**ataflow-centric **I**ntegrated **S**imulation **O**f **L**arge scale **V**ANETs.

### What is Disolv?

Disolv is a VANET simulator capable of studying futuristic Intelligent Transportation System (ITS) applications. 
Disolv is designed with the primary goal of supporting large-scale simulations of various ITS scenarios.

---
<details>
    <summary>
        <b>Introduction</b>
    </summary>

#### What is a VANET?

**V**ehicular **A**d-hoc **NET**work (VANET) is a system of vehicles equipped with communication devices.
Using the communication equipment, vehicles exchange information among themselves and with the traffic infrastructure.
This enables an entire ecosystem of traffic safety and comfort applications called Intelligent Transporation System (ITS) applications.


---

#### What is a VANET simulator?

Initial validation of ITS applications is carried out through VANET simulations.
Due to the scale, cost and the safety concerns involved in live testing, VANET simulations are extensively used as a playground before validating the application in field trials.
[Veins](https://veins.car2x.org/) and [Eclipse MOSAIC](https://eclipse.dev/mosaic/) are some of the popular open-source VANET simulators.

</details>

---

### Crates

Disolv is modularized to support easier extension development.
The functionality is arranged in a hierarchial onion-style architecture.

---

<details>
    <summary>
        <b>Crates</b>
    </summary>

#### Core

Core contains the agent scheduler and the terminal UI implementation.
Using newtype pattern of rust, several primitives are defined for the rest of the simulator to use.
All the essential traits are also declared here.

#### Models

A definite implementation for some of the basic traits are provided in this crate.
Further, the device behavior models are designed here to be independent of the device type.
Model parameterization is supported.
If a new requirement arises, users can define their own models in this crate.
By following the traits for the models, it is easy to make the model be compatible with the simulator.


#### Input

Parquet files are used to read the simulation input from the disk. 
Expansion of support to read other file formats is in the pipeline.


#### Output

All the output data is written in the form of parquet files, which can be further processed by user's preferred tools.
Expansion of support to write other file formats is in the pipeline.

</details>

---

### Installation

_coming soon_

--- 

### Sample Scenarios

_coming soon_

--- 

### Publication

Please cite the following article if you used Disolv in your research.

```
@inproceedings{tangirala2024simulation,
    author = {Tangirala, Nagacharan Teja and Sommer, Christoph and Knoll, Alois},
    title = {{Simulating Data Flows of Very Large Scale Intelligent Transportation Systems}},
    booktitle = {2024 ACM SIGSIM International Conference on Principles of Advanced Discrete Simulation (SIGSIM-PADS 2024)},
    addendum = {(to appear -- author's version available at \url{https://syncandshare.lrz.de/getlink/fiUydMpwAGk57a1vufokvG/PADS_camera_ready.pdf})},
    address = {Atlanta, GA},
    month = Jun,
    publisher = {ACM},
    year = {2024},
}
```

--- 

### Acknowledgements
---

This project is not possible without the following communities - 

[krABMaga](https://krabmaga.github.io/) \
[KD-tree](https://github.com/sdd/kiddo)


