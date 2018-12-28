#! /usr/bin/env python
# encoding: utf-8

from __future__ import unicode_literals

from setuptools import setup, find_packages

setup(
    name="fst_test_data",
    version="0.1.0",
    description="Python test generator for Rustfst",
    author="Alexandre Caulier",
    author_email="alexandre.caulier@protonmail.com",
    classifiers=[
        "Programming Language :: Python :: 2",
        "Programming Language :: Python :: 2.7",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.4",
        "Programming Language :: Python :: 3.5",
        "Programming Language :: Python :: 3.6",
    ],
    install_requires=["pynini==1.6"],
    packages=find_packages(),
    include_package_data=True,
    zip_safe=False,
)
