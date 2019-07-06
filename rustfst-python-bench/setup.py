#!/usr/bin/python
# -*- coding: utf-8 -*-

from setuptools import setup, find_packages

packages = find_packages()

install_requires = []

setup(
    name="rustfst_python_bench",
    description="Python package to benchmark openfst CLI against rustfst CLI",
    version="0.1.0",
    author="Alexandre Caulier",
    author_email="alexandre.caulier@protonmail.com",
    license="All rights reserved",
    install_requires=install_requires,
    packages=packages,
    include_package_data=True,
    zip_safe=False,
)