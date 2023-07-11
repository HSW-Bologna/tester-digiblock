#!/usr/bin/python

import os

import argparse
import yaml


def processLogs(dataDir):
    reports = []
    files = []
    path = dataDir
    # print("Path: ", path)
    report = {}
    for r, d, f in os.walk(path):
        for file in f:
            if file.lower().find('.yaml') != -1:
                files.append(os.path.join(r, file))
                with open(os.path.join(r, file), 'r') as fi:
                    report = yaml.safe_load(fi)
                    reports.append(report)
                fi.close()
    return reports


def TestData_Validator(testdata):

    if testdata["formato"] != 1:
        raise RuntimeError(testdata["formato"], ": unsupported data version")

    listaCollaudo = ["attrezzatura", "applicazione", "versione", "codice_dut", "istanza", "stazione",
                     "firmware", "matricola", "data", "ora", "durata", "operatore", "esito", "codice_di_errore", "note"]
    listaGenerica = ["formato", "collaudo", "prove" ]
    listaProve = ["prova", "descrizione", "esito",
                  "durata", "udm", "valore", "minimo", "massimo"]

    for l in listaGenerica:
        if l not in testdata:
            raise RuntimeError(l, " missed key")
    for l in listaCollaudo:
        if l not in testdata["collaudo"]:
            raise RuntimeError(l, " missed key")

    for x in testdata:
        if x not in listaGenerica:
            raise RuntimeError(x, " key not in version 1")

    for x in testdata["collaudo"]:
        if x not in listaCollaudo:
            raise RuntimeError(x, " key not in version 1")

    for l in listaProve:
        for prova in testdata["prove"]:
            if l not in prova:
                raise RuntimeError(l, " missed key")

    for prova in testdata["prove"]:
        for chiave in prova:
            if chiave not in listaProve:
                raise RuntimeError(chiave, " key not in version 1")


def main():

    # Initiate the parser
    parser = argparse.ArgumentParser()

    # Add long and short argument
    parser.add_argument("--dir", "-d", help="Files folder")

    # Read arguments from the command line
    args = parser.parse_args()

    if args.dir:
        print("Set pathfile path  to%s" % args.dir)

    try:
        print("----------------------------------------------")
        print("STARTED\n\r")

        values = processLogs(args.dir)
        for testdata in values:
            TestData_Validator(testdata)

        print("*********************\n\r")
        print("valid source data\n\r")
        print("*********************\n\r")

        print("\n\rDONE")
        print("----------------------------------------------")
    except:
        raise
    finally:
        pass


if __name__ == "__main__":
    main()
