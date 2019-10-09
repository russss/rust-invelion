# invelion

This driver provides Rust support for a number of Chinese-manufactured UHF RFID
Gen2 reader modules based on the Impinj Indy R2000 RF chipset with an AVR ARM processor.

It appears that these readers are based on a white-label module design which is used by a
number of Chinese manufacturers. The original designer remains unknown. I've named this module
"invelion" as this is the device I have.

### Example Code

Examples of the use of this library can be found in the `examples` directory.

### Supported Readers

Unless otherwise noted, the modules listed below are *not tested* with this library, but are
suspected to use the same protocol due to visual similarity, similarity to modules from the
same manufacturer, or availability of manuals (mostly on the FCC website) depicting the same
evaluation software.

Please let me know if you get a new reader working with this code!

[Invelion/INNOD](http://www.innod-rfid.net/) (Shenzhen Invelion Technology CO., Ltd):
  * IND905 / YR905 / IND901 / YR901 (*tested and working*)
  * YR900
  * IND904
  * IND9010 (suspected identical to Rodinbell D100)
  * IND9051

[Rodinbell](http://www.rodinbell.com/) (Shenzhen Rodinbell Technology CO., Ltd):
  * D100 ([FCC](https://fcc.io/2AKQD-D100))
  * M500 ([FCC](https://fcc.io/2AKQD-M500))
  * M2800 ([FCC](https://fcc.io/2AKQD-M2800))
  * M2600 ([FCC](https://fcc.io/2AKQD-M2600))
  * M2900 ([FCC](https://fcc.io/2AKQD-M2900))
  * S-8600 ([FCC](https://fcc.io/2AKQD-S-8600A))
  * S-8800


License: LGPL-3.0-or-later
