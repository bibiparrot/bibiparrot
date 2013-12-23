'''


'''

import sys, os , time, inspect, imp, platform, logging
import wx
import ConfigParser
import unittest

from ...bibiparrot.Utils.utils import *
from ...bibiparrot.Constants.constants import *
from ...bibiparrot.Configurations.Configuration import Configuration, log

class TestConfiguration(unittest.TestCase):

    def setUp(self):
        self.conf = Configuration()
        self.conf_file = open("TestConfiguration.ini", "w")
        self.conf.CONF_FILE = self.conf_file.name
        self.conf_file.write("# Encoding=UTF-8\n")
        self.conf_file.write("# Tips=Please Open This File by Encoding Type &Encoding in the First Line.]\n")
        self.conf_file.write("[default]\n")
        self.conf_file.write("[helloworld.append]\n")
        self.conf_file.write("Loads=\n")
        self.conf_file.write("PythonFile=examples/helloworld.py\n")
        self.conf_file.write("Method=append\n")
        self.conf_file.write('Args="hello", "worlds"\n')
        self.conf_file.write("Return=example_hello_world\n")
        self.conf_file.write("Dumps=example_hello_world:helloworld.append.example_hello_world\n")
        self.conf_file.write("\n")
        self.conf_file.close()

    def tearDown(self):
        # with open (self.conf_file.name, "r") as conf_file:
        #     data = conf_file.readlines()
#         print ""
#         for line in data:
#             print line.replace("\n","")
        os.remove(self.conf.CONF_FILE)
#         print self.conf.dump()

    def test_getConf(self):
        val = self.conf.getConf("helloworld.append", "Args")
        self.assertEqual(val, '"hello", "worlds"')
        val = self.conf.getConf("helloworld.append", "Loads")
        self.assertEqual(val, "")
        val = self.conf.getConf("helloworld.append", "Argss")
        self.assertEqual(val, None)

    def test_setConf(self):
        self.conf.setConf("helloworld.append", "Argsss", "TEST")
        val = self.conf.getConf("helloworld.append", "Argsss")
        self.assertEqual(val, "TEST")
        log().debug("%s: val=%s", "s",  "s")
        log().debug("%s: val=%s", funcname(),  val)


if __name__ == '__main__':
    unittest.main()


