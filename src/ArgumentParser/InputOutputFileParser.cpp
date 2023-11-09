#include "InputOutputFileParser.hpp"
#include <stdexcept>

std::string InputOutputFileParser::input() {
  return this->m_input;
}

std::string InputOutputFileParser::output() {
  return this->m_output;
}

bool InputOutputFileParser::submit(std::string *part) {
  // Ignore empty arguments; there's no situation
  // here that we could parse an argument like that.
  if (part->empty()) {
    return false;
  }

  switch (state) {
    case State::InputFile:
      if (!this->m_input.empty()) {
        throw std::runtime_error("Input path already specified.");
      }

      this->m_input = *part;
      this->state = State::None;

      return true;

    case State::OutputFile:
      if (!this->m_output.empty()) {
        throw std::runtime_error("Output path already specified.");
      }

      this->m_output = *part;
      this->state = State::None;

      return true;

    case State::None:
      if (*part == "--input" || *part == "-i") {
        this->state = State::InputFile;

        return true;
      }

      if (*part == "--output" || *part == "-o") {
        this->state = State::OutputFile;

        return true;
      }

      if (this->m_input.empty() && part->at(0) != '-') {
        this->m_input = *part;

        return true;
      }

      return false;
  }

  // This isn't possible, but G++ doens't know any better
  return false;
}

void InputOutputFileParser::finalize() {
  if (this->m_output.empty()) {
    this->m_output = "program";
  }

  if (this->m_input.empty()) {
    throw std::runtime_error("No input file specified.");
  }
}
