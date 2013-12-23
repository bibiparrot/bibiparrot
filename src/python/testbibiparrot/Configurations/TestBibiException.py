
import logging

import unittest

from ...bibiparrot.Configurations.BibiException import BibiException
from ...bibiparrot.Utils.utils import *

class TestBibiException(unittest.TestCase):
    def setUp(self):
        pass

    def tearDown(self):
        pass

    def testBibi(self):
        try:
            error = "TestBibiException ...."
            raise BibiException(error)
        except BibiException as bibi:
            self.assertEqual(error, bibi.message, "BibiException Fail.")


if __name__ == '__main__':
    unittest.main()