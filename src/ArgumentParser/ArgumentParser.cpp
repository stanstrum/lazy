#include "ArgumentParser.hpp"

#include <optional>
#include <sstream>

void parse_arguments_with_parsers(char **argv, const std::vector<ArgumentParser*> parsers) {
  std::optional<size_t> parser_idx;

  // We start at 1 here because argv[0] is the program name.
  size_t argv_idx = 1;

  // Very C.  This is just a simple iteration over the parts
  // of an array of C-strings, ending when we find a NULL
  // value.
  for (char *part_ptr; (part_ptr = *(argv + argv_idx)); argv_idx++) {
    std::string part = part_ptr;

    // If there is a parser that is currently accepting values,
    // submit it.
    if (parser_idx.has_value()) {
      bool consumed = parsers[parser_idx.value()]->submit(&part);

      // If it consumed this argument, skip over the rest
      // of the loop that looks for a new parser.
      if (consumed) {
        continue;
      }
    }

    // We arrive here if we don't have a parser to
    // submit the argument to.  Find a new one.
    for (size_t i = 0; i < parsers.size(); i++) {
      // Don't resubmit the same part to the same parser
      // if it didn't consume it.
      if (parser_idx.has_value() && parser_idx.value() == i) {
        // Clear it after we've skipped it.
        // Since we assign it later only if it was found,
        // if it is still empty after this loop, that means
        // no parser was able to accept it.  Then we can
        // throw an exception.
        parser_idx.reset();

        continue;
      }

      // Submit the argument and save the resulting boolean
      // of whether it was consumed.
      bool consumed = parsers[i]->submit(&part);

      if (consumed) {
        // Save the index of the parser that is
        // now accepting the arguments.
        parser_idx = i;

        break;
      }
    }

    // If parser_idx is still empty, no parser accepted the argument.
    // Therefore we throw an error.
    if (!parser_idx.has_value()) {
      std::stringstream error;
      error << "Could not parse argument: " << part;

      throw std::runtime_error(error.str());
    }
  }

  // We exhausted argv of arguments.  Inform all parsers
  // of that so that they can validate the arguments,
  // set defaults, etc.
  for (auto *parser : parsers) {
    parser->finalize();
  }
}
