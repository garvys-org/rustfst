import subprocess
import os
import datetime
import platform


def check_property_set(path_openfst_bins, prop_name, path_fst):
    path_bin = os.path.join(path_openfst_bins, "fstinfo")

    subprocess.check_call([f"{path_bin} --info_type=\"long\" {path_fst} | grep \"{prop_name}\""
                           f" | awk '{{print $NF}}' | "
                           f"(read p1; python rustfst-python-bench/rustfst_python_bench/lol.py $p1) "], shell=True)


def header_report(report_f, path_in_fst):
    report_f.write(f"**Input FST** : \n")
    report_f.write(f"- Path : {path_in_fst}\n")
    report_f.write(f"- Size : {os.path.getsize(path_in_fst) * 10.0**(-6):.2f} MB\n\n")
    report_f.write(f"**Date**: {datetime.datetime.now().strftime('%x %X')}\n\n")
    report_f.write(f"**Computer specs**:\n")
    report_f.write(f"- Machine type : {platform.machine()}\n")
    report_f.write(f"- Platform : {platform.platform()}\n")
    report_f.write(f"- Processor : {platform.processor()}\n")
    report_f.write(f"- System : {platform.system()}\n")
