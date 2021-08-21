#!/usr/bin/env python3

import datetime
import os
import shutil
import subprocess
import time

import pandas as pd
import psutil


def main():

    if not os.path.samefile(os.getcwd(), os.path.dirname(__file__)):
        raise Exception('Must be run from source file directory')

    # TODO
    #ensure_users()
    #ensure_processes()
    #ensure_turbo_off()

    df = pd.DataFrame()

    df = df.append(build_unenclaved(), sort=True)
    df = df.append(build_enclaved(), sort=True)

    for i in range(5):
        unenclaved_result = run_minimal_startup(False)
        df = df.append(unenclaved_result, sort=True)
        enclaved_result = run_minimal_startup(True)
        df = df.append(enclaved_result, sort=True)

    print(df)

    print_msg('Writing serialized result to "measurement_minimal_startup.tsv"')
    df.to_csv('measurement_minimal_startup.tsv', sep='\t')


def ensure_users():

    who = subprocess.run(['who'], check=True, stdout=subprocess.PIPE)
    if who.stdout.count(b'\n') != 1:
        raise Exception('Too many logged in users')


def ensure_uptime():

    uptime = time.time() - psutil.boot_time()
    if uptime > 86400:
        raise Exception('Host not recently rebooted')


def ensure_processes():

    for p in psutil.process_iter():
        p_name = p.name()
        if 'cron' in p_name:
            raise Exception('cron must be stopped')
        if 'libvirtd' in p_name:
            raise Exception('libvirtd must be stopped')
        if 'snapd' in p_name:
            raise Exception('snapd must be stopped')
        if 'systemd-nspawn' in p_name:
            raise Exception('No systemd-nspawn processes must be running')
        if 'Runner.Listener' in p_name:
            raise Exception('github-actions-runner must be stopped')
        if p_name == 'sh' and 'vscode-server' in p.cmdline()[1]:
            raise Exception('No vscode-server processes must be running')

    # See https://unix.stackexchange.com/q/473992
    active_timers = subprocess.run(['systemctl', 'list-units', '--type=timer', '--state=active', '--no-legend'],
                                   check=True, stdout=subprocess.PIPE)
    if active_timers.stdout.count(b'\n') != 0:
        raise Exception('Found active systemd timers')

    # TODO: Adjust number
    if len(psutil.pids()) > 25:
        raise Exception('Too many processes running on system')


def ensure_turbo_off():

    with open('/sys/devices/system/cpu/intel_pstate/no_turbo') as f:
        no_turbo = f.read().rstrip()

    if no_turbo != '1':
        raise Exception('Turbo boost must be disabled')


def build_unenclaved():

    print_msg('Building unenclaved program in debug and release mode')
    os.symlink('Cargo.unenclaved.toml', 'Cargo.toml')

    debug_multiunit_cmd = ['cargo', 'build', '--features=enclavization_bin']
    debug_multiunit_result = {
        'program': 'build minimal_startup',
        'operation': 'debug',
        'enclaved': False,
        'cmd': debug_multiunit_cmd,
        'descr': 'Time spent building the program in debug mode with multiple codegen units'
    }
    debug_1unit_cmd = ['cargo', 'rustc', '--features=enclavization_bin', '--', '-C', 'codegen-units=1']
    debug_1unit_result = {
        'program': 'build minimal_startup',
        'operation': 'debug, codegen-units=1',
        'enclaved': False,
        'cmd': debug_1unit_cmd,
        'descr': 'Time spent building the program in debug mode with one codegen unit'
    }
    release_multiunit_cmd = ['cargo', 'build', '--features=enclavization_bin', '--release']
    release_multiunit_result = {
        'program': 'build minimal_startup',
        'operation': 'release',
        'enclaved': False,
        'cmd': release_multiunit_cmd,
        'descr': 'Time spent building the program in release mode with multiple codegen units'
    }
    release_1unit_cmd = ['cargo', 'rustc', '--features=enclavization_bin', '--release', '--',
                         '-C', 'codegen-units=1']
    release_1unit_result = {
        'program': 'build minimal_startup',
        'operation': 'release, codegen-units=1',
        'enclaved': False,
        'cmd': release_1unit_cmd,
        'descr': 'Time spent building the program in release mode with one codegen unit'
    }

    try:
        shutil.rmtree('target', ignore_errors=True)
        begin = time.time()
        subprocess.run(debug_multiunit_cmd, check=True)
        debug_multiunit_result['duration'] = (time.time() - begin) * 1000
        debug_multiunit_result['time'] = datetime.datetime.now()

        shutil.rmtree('target', ignore_errors=True)
        begin = time.time()
        subprocess.run(debug_1unit_cmd, check=True)
        debug_1unit_result['duration'] = (time.time() - begin) * 1000
        debug_1unit_result['time'] = datetime.datetime.now()

        shutil.rmtree('target', ignore_errors=True)
        begin = time.time()
        subprocess.run(release_multiunit_cmd, check=True)
        release_multiunit_result['duration'] = (time.time() - begin) * 1000
        release_multiunit_result['time'] = datetime.datetime.now()

        shutil.rmtree('target', ignore_errors=True)
        begin = time.time()
        subprocess.run(release_1unit_cmd, check=True)
        release_1unit_result['duration'] = (time.time() - begin) * 1000
        release_1unit_result['time'] = datetime.datetime.now()

        return [debug_multiunit_result, debug_1unit_result, release_multiunit_result, release_1unit_result]
    finally:
        os.remove('Cargo.toml')


def build_enclaved():

    print_msg('Building enclaved program in debug and release mode')

    cmd = ['make', 'all']
    debug_result = {
        'program': 'build minimal_startup',
        'operation': 'debug, codegen-units=1',
        'enclaved': True,
        'cmd': cmd,
        'descr': 'Time spent building the program in debug mode'
    }
    release_result = {
        'program': 'build minimal_startup',
        'operation': 'release, codegen-units=1',
        'enclaved': True,
        'cmd': cmd,
        'descr': 'Time spent building the program in release mode'
    }

    subprocess.run(['make', 'clean'], check=True)
    begin = time.time()
    subprocess.run(cmd, check=True)
    debug_result['duration'] = (time.time() - begin) * 1000
    debug_result['time'] = datetime.datetime.now()

    subprocess.run(['make', 'clean'], check=True)
    begin = time.time()
    subprocess.run(cmd, env=dict(os.environ, BUILD_MODE='release'), check=True)
    release_result['duration'] = (time.time() - begin) * 1000
    release_result['time'] = datetime.datetime.now()

    return [debug_result, release_result]


def run_minimal_startup(enclaved):

    if enclaved:
        minimal_startup_path = 'build/minimal_startup'
        print_msg('Running enclaved minimal_startup')
    else:
        minimal_startup_path = 'target/release/minimal_startup'
        print_msg('Running unenclaved minimal_startup')

    cmd = [minimal_startup_path]
    result = {
        'program': 'minimal_startup',
        'enclaved': enclaved,
        'cmd': cmd,
        'descr': 'Time of a complete program execution, as watched from outside',
        'time': datetime.datetime.now()
    }
    begin = time.time()
    subprocess.run(cmd, check=True)
    result['duration'] = (time.time() - begin) * 1000

    return [result]


def print_msg(message):

    print('>>> {}'.format(message))


if __name__ == '__main__':

    main()
