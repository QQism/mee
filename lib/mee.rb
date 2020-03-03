require 'mee/version'
require 'rutie'

module Mee
  Rutie.new(:mee_console).init 'Init_mee', __dir__

  class Console
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
    end
  end
end
