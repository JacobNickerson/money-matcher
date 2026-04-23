<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a id="readme-top"></a>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]



<!-- PROJECT LOGO -->
<br />
<div align="center">
  <a href="https://github.com/JacobNickerson/money-matcher">
    <img src="resources/logo.png" alt="Logo" width="80" height="80">
  </a>

<h3 align="center">MatchMaker</h3>

  <p align="center">
    An exchange simulator and strategy backtester
    <br />
    <a href="https://github.com/JacobNickerson/money-matcher/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    &middot;
    <a href="https://github.com/JacobNickerson/money-matcher/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a></li>
      </ul>
    </li>
    <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li>
    <li><a href="#usage">Usage</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li>
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project

[![Product Name Screen Shot][product-screenshot]](https://example.com)

Modern financial markets rely on High-Frequency Trading (HFT), yet the field remains inaccessible to students due to extreme financial risk and the high cost of proprietary software. We present an educational HFT system designed for risk-free experimentation. The platform features a high-performance Rust matching engine capable of processing hundreds of thousands of orders per second, paired with a configurable data simulation server. By integrating industry-standard protocols like FIX and ITCH with a user-friendly Python API, the system provides a realistic environment for students to develop, backtest, and visualize algorithmic strategies under various market conditions.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



### Built With

* [![Rust][Rust]][Rust-url]
* [![Python][Python]][Python-url]
* [![Qt][Qt]][Qt-url]

<p align="right">(<a href="#readme-top">back to top</a>)</p>

<!-- GETTING STARTED -->
## Getting Started

Most of the build process is automated through the provided Makefile. However, there are some dependencies that must be handled first.

### Prerequisites

These dependencies are available for all available operating systems. For demonstration, installation is shown on Fedora linux, however adjusting this for other operating systems shouldn't be difficult.

* Python 3.10
  ```sh
  sudo dnf install python310
  ```
* Qt5
  ```sh
  sudo dnf install qt5-devel
  ```
* Cargo
  ```sh
  sudo dnf install cargo
  ```
* C++ stdlib
  ```sh
  # Install whatever C++ compiler you like
  sudo dnf install gcc-c++ 
  ```
* Make
  ```sh
  sudo dnf install make
  ```
Optionally, `uv` can be used to manage Python runtimes. The project was developed on Python 3.10 and may or may not work on other versions.
* uv
  ```sh
  sudo dnf install uv
  ```
A one-shot:
```sh
  sudo dnf install python310 qt5-devel cargo gcc-c++ make uv
```

### Installation

1. Install the dependencies listed above
2. Optionally, create and activate a virtual environment for managing python dependencies
   ```sh
   python3 -m venv .venv && source .venv/bin/activate
   ```
3. Run the build script, it will handle installing python dependencies, compilation of the server and python shared lib, and installation of the python shared lib into the current python environment
   ```sh
   make
   ```

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- USAGE EXAMPLES -->
## Usage

Use this space to show useful examples of how a project can be used. Additional screenshots, code examples and demos work well in this space. You may also link to more resources.

_For more examples, please refer to the [Documentation](https://example.com)_

<p align="right">(<a href="#readme-top">back to top</a>)</p>

See the [open issues](https://github.com/JacobNickerson/money-matcher/issues) for a full list of proposed features (and known issues).

<p align="right">(<a href="#readme-top">back to top</a>)</p>



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>

### Top contributors:

<a href="https://github.com/JacobNickerson/money-matcher/graphs/contributors">
  <img src="https://contrib.rocks/image?repo=JacobNickerson/money-matcher" alt="contrib.rocks image" />
</a>



<!-- ACKNOWLEDGMENTS -->
## Acknowledgments

* Special thanks to Ashish Aggarwal, who advised our project

<p align="right">(<a href="#readme-top">back to top</a>)</p>



[contributors-shield]: https://img.shields.io/github/contributors/JacobNickerson/money-matcher.svg?style=for-the-badge
[contributors-url]: https://github.com/JacobNickerson/money-matcher/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/JacobNickerson/money-matcher.svg?style=for-the-badge
[forks-url]: https://github.com/JacobNickerson/money-matcher/network/members
[stars-shield]: https://img.shields.io/github/stars/JacobNickerson/money-matcher.svg?style=for-the-badge
[stars-url]: https://github.com/JacobNickerson/money-matcher/stargazers
[issues-shield]: https://img.shields.io/github/issues/JacobNickerson/money-matcher.svg?style=for-the-badge
[issues-url]: https://github.com/JacobNickerson/money-matcher/issues
[product-screenshot]: resources/screenshot.png
[Rust]: https://img.shields.io/badge/Rust-%23000000.svg?e&logo=rust&logoColor=white
[Rust-url]: https://rust-lang.org
[Python]: https://img.shields.io/badge/Python-3776AB?logo=python&logoColor=fff
[Python-url]: https://www.python.org
[Qt]: https://img.shields.io/badge/Qt-2CDE85?logo=Qt&logoColor=fff
[Qt-url]: https://www.qt.io
