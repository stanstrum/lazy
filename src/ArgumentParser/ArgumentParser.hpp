#include <string>
#include <vector>

/**
 * @brief The interface that @ref parse_arguments_with_parsers
 * uses to parse arguments.
 */
class ArgumentParser {
public:
  /**
   * @brief Submit an argument to the parser.
   *
   * This method should take an argument from argv
   * and return a boolean corresponding to whether
   * or not it was consumed.  If we don't consume it,
   * the same argument will not be resubmitted to us.
   *
   * @param part The current C-string argument from argv.
   *
   * @return `true` This item was parsed and used; seek to
   * the next argument and submit it to us.
   *
   * @return `false` This item was not parsed; another parser
   * should take over.
   *
   * @throws runtime_error If parsing of this argument fails.
   */
  virtual bool submit(std::string *part) = 0;

  /**
   * @brief Finalize the parsing of arguments.
   *
   * Not all parsers necessarily need to do this,
   * however parses can use this method to validate
   * parsed values, detect duplicate/missing
   * arguments, set defaults, etc.
   *
   * @throws runtime_error If validation/finalization fails.
   */
  virtual void finalize() = 0;
};

/**
 * @brief Parse argv with the provided parsers.
 *
 * This function takes in argv and a vector initializer
 * of pointers to parsers.  For each argument, the parsers
 * will be queried whether they can parse the current argv item.
 * If so, it is consumed and the parser will be asked first
 * for the next argument until it returns false.  Then, a
 * new parser will be found to parse the arguments, otherwise
 * an exception will be thrown.  Finally, each parser will be
 * informed that the parsing has finished.
 *
 * Afterwards, you can retrieve the parsed data from
 * your parsers by their respective interfaces.
 *
 * @note This function queries the parsers from left to
 * right in the list.  A catch-all that ignores
 * unrecognized arguments should appear at the end of the
 * argument vector.
 *
 * @throws runtime_error If at any point an argument cannot be parsed.
 *
 * @param argv argv from main.
 * @param parsers The vector of pointers to class instances that implement
 * @ref ArgumentParser.
 */
void parse_arguments_with_parsers(char **argv, const std::vector<ArgumentParser*> parsers);
