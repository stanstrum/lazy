#include <iostream>

#include "ArgumentParser/InputOutputFileParser.hpp"

int main(int argc, char **argv) {
  InputOutputFileParser io_file_parser;

  try {
    parse_arguments_with_parsers(argv, std::vector<ArgumentParser*> { &io_file_parser });
  } catch (const std::runtime_error &error) {
    std::cerr << error.what() << std::endl;

    return 1;
  }

  std::cout << "IO File Parser:" << std::endl;
  std::cout << "  Input file  : " << io_file_parser.input() << std::endl;
  std::cout << "  Output file : " << io_file_parser.output() << std::endl;

  return 0;
}
