import datetime
import io
import os
import platform
import subprocess
import tempfile

from rustfst_python_bench.constants import OPENFST_BINS


def check_property_set(path_fst, prop):
    cli = os.path.join(OPENFST_BINS, "fstinfo")

    with tempfile.TemporaryDirectory() as tmpdirname:
        path_info_data = os.path.join(tmpdirname, "info.txt")
        cmd = f"{cli} {path_fst} | grep '{prop}'  > {path_info_data} "

        subprocess.check_call([cmd], shell=True)

        with io.open(path_info_data, mode="r") as f:
            data = f.read().strip()

            if data[-1] != "y":
                raise RuntimeError(f"Expected prop '{prop}' to be set to 'y' but found '{data[-1]}'")


def check_fst_equals(path_fst_1, path_fst_2):
    cli = os.path.join(OPENFST_BINS, "fstequal")

    cmd = f"{cli} {path_fst_1} {path_fst_2}"

    subprocess.check_call([cmd], shell=True)


def header_report(report_f, path_in_fst, n_warm_ups, n_runs):
    report_f.write(f"**Bench parameters** :\n")
    report_f.write(f"- Num warmup rounds : {n_warm_ups}\n")
    report_f.write(f"- Num bench runs : {n_runs}\n\n")
    report_f.write(f"**Input FST** : \n")
    report_f.write(f"- Path : {path_in_fst}\n")
    report_f.write(f"- Size : {os.path.getsize(path_in_fst) * 10.0**(-6):.2f} MB\n\n")
    report_f.write(f"**Date**: {datetime.datetime.now().strftime('%x %X')}\n\n")
    report_f.write(f"**Computer specs**:\n")
    report_f.write(f"- Machine type : {platform.machine()}\n")
    report_f.write(f"- Platform : {platform.platform()}\n")
    report_f.write(f"- Processor : {platform.processor()}\n")
    report_f.write(f"- System : {platform.system()}\n")
