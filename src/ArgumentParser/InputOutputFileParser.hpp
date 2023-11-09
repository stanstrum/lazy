#include "ArgumentParser.hpp"

/**
 * @brief A class that implements @ref ArgumentParser
 * and parses the input file and output file for compilation.
 *
 * Additionally, it will implicitly parse an input path without -i,
 * as well as returning an error for duplicate definitions.
 */
class InputOutputFileParser: public ArgumentParser {
private:
  /**
   * @brief The currently parsed input file path.
   */
  std::string m_input;

  /**
   * @brief The currently parsed input file path.
   */
  std::string m_output;

  /**
   * @brief What this parser is looking for next.
   */
  enum State {
    /**
     * @brief Nothing in particular; could be
     * a switch, could be an implicit input file.
     */
    None,
    /**
     * @brief An input file path.  This state occurs
     * after the input switch is explicitly specified.
     */
    InputFile,
    /**
     * @brief An output file path.  This state occurs
     * after the output switch is explicitly specified.
     */
    OutputFile,
  };

  /**
   * @brief The current parser state.
   */
  State state = State::None;

public:
  /**
   * @brief Get @ref InputOutputFileParser::m_input.
   *
   * @return std::string The parsed input file path.
   */
  std::string input();

  /**
   * @brief Get @ref InputOutputFileParser::m_input.
   *
   * @return std::string The parsed output file path.
   */
  std::string output();

  /**
   * @copydoc ArgumentParser::submit(std::string*)
   */
  bool submit(std::string *part);

  /**
   * @copydoc ArgumentParser::finalize()
   */
  void finalize();
};
