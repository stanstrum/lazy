#include "ArgumentParser.hpp"

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
   * @enum InputOutputFileParser::State
   * @brief What this parser is looking for next.
   *
   * @var State::None
   * @brief Nothing in particular; could be
   * a switch, could be an implicit input file.
   *
   * @var State::InputFile
   * @brief An input file path.  This state occurs
   * after the input switch is explicitly specified.
   *
   * @var State::OutputFile
   * @brief An output file path.  This state occurs
   * after the output switch is explicitly specified.
   */
  enum State {
    None,
    InputFile,
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
