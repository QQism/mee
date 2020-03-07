require 'mee/version'
require 'rutie'

module Mee
  Rutie.new(:mee_console).init 'Init_mee', __dir__

  class Console
    KEY_CONSTANTS = 'constants'.freeze
    KEY_GLOBAL_VARIABLES = 'global_variables'.freeze
    KEY_LOCAL_VARIABLES = 'local_variables'.freeze
    KEY_INSTANCE_VARIABLES = 'instance_variables'.freeze
    KEY_METHODS = 'methods'.freeze
    KEY_PRIVATE_METHODS = 'private_methods'.freeze

    def initialize
      MeeConsole.start()
    end

    class << self
      def local_binding
        @local_binding ||= eval("self.class.send(:remove_method, :irb_binding) if defined?(irb_binding); private; def irb_binding; binding; end; irb_binding",
          TOPLEVEL_BINDING,
          __FILE__,
          __LINE__ - 3)
      end

      def evaluate(str)
        eval(str, local_binding).to_s
      end

      def initial_suggestions
        suggestions = {}

        Object.constants.each_with_object(suggestions) do |name, hash|
          hash[name] = ['constant']
        end

        # global variables
        eval(KEY_GLOBAL_VARIABLES, local_binding).each_with_object(suggestions) do |name, hash|
          hash[name] = [KEY_GLOBAL_VARIABLES]
        end

        # local variables
        eval(KEY_LOCAL_VARIABLES, local_binding).each_with_object(suggestions) do |name, hash|
          hash[name] = [KEY_LOCAL_VARIABLES]
        end

        # instance variables
        eval(KEY_INSTANCE_VARIABLES, local_binding).each_with_object(suggestions) do |name, hash|
          hash[name] = [KEY_INSTANCE_VARIABLES]
        end

        # methods
        eval(KEY_METHODS, local_binding).each_with_object(suggestions) do |name, hash|
          hash[name] = [KEY_METHODS]
        end

        # private methods
        eval(KEY_PRIVATE_METHODS, local_binding).each_with_object(suggestions) do |name, hash|
          hash[name] = [KEY_PRIVATE_METHODS]
        end

        suggestions
      end

      def context_suggestions
      end
    end
  end
end
